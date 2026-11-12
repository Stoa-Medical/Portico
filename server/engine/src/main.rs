mod db;
mod runner;
mod services;
mod steps;

use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

const DRAIN_TIMEOUT: Duration = Duration::from_secs(30);
const LEASE_SWEEP_INTERVAL: Duration = Duration::from_secs(30);
const LEASE_MARGIN_SECONDS: u64 = 30;

struct Config {
    database_url: String,
    health_port: u16,
    poll_interval: Duration,
    max_concurrent: usize,
    lease_seconds: u64,
}

impl Config {
    fn from_env() -> Result<Self> {
        let lease_seconds: u64 = env_or("SIGNAL_LEASE_SECONDS", 300)?;
        anyhow::ensure!(
            lease_seconds > LEASE_MARGIN_SECONDS,
            "SIGNAL_LEASE_SECONDS must exceed {LEASE_MARGIN_SECONDS}"
        );
        Ok(Self {
            database_url: env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            health_port: env_or("HEALTH_PORT", 8080)?,
            poll_interval: Duration::from_millis(env_or("POLL_INTERVAL_MS", 1000)?),
            max_concurrent: env_or("MAX_CONCURRENT_SIGNALS", 8)?,
            lease_seconds,
        })
    }

    /// A run is cut off before its lease can expire, so the sweep only ever
    /// reclaims leases left by a crashed engine.
    fn run_budget(&self) -> Duration {
        Duration::from_secs(self.lease_seconds - LEASE_MARGIN_SECONDS)
    }
}

fn env_or<T: FromStr>(name: &str, default: T) -> Result<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match env::var(name) {
        Ok(raw) => raw.parse().with_context(|| format!("invalid {name}")),
        Err(_) => Ok(default),
    }
}

#[derive(Clone)]
struct HealthState {
    draining: Arc<AtomicBool>,
}

async fn health_check(State(state): State<HealthState>) -> impl IntoResponse {
    if state.draining.load(Ordering::SeqCst) {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "draining" })),
        )
    } else {
        (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
    }
}

async fn poll_signals(
    pool: PgPool,
    config: Arc<Config>,
    cancel: CancellationToken,
    tracker: TaskTracker,
) {
    let permits = Arc::new(Semaphore::new(config.max_concurrent));
    let mut sweep = tokio::time::interval(LEASE_SWEEP_INTERVAL);

    loop {
        let permit = tokio::select! {
            _ = cancel.cancelled() => return,
            _ = sweep.tick() => {
                match db::release_leases(&pool, true).await {
                    Ok(0) => {}
                    Ok(n) => println!("[engine] reclaimed {n} expired lease(s)"),
                    Err(e) => eprintln!("[engine] lease sweep failed: {e:#}"),
                }
                continue;
            }
            permit = permits.clone().acquire_owned() => permit.expect("semaphore is never closed"),
        };

        match db::lease_next(&pool, config.lease_seconds as f64).await {
            Ok(Some(signal)) => {
                let pool = pool.clone();
                let budget = config.run_budget();
                tracker.spawn(async move {
                    let signal_id = signal.id;
                    if let Err(e) = runner::process(&pool, signal, budget).await {
                        eprintln!("[engine] signal {signal_id} left dispatched: {e:#}");
                    }
                    drop(permit);
                });
            }
            Ok(None) => {
                drop(permit);
                tokio::select! {
                    _ = cancel.cancelled() => return,
                    _ = tokio::time::sleep(config.poll_interval) => {}
                }
            }
            Err(e) => {
                drop(permit);
                eprintln!("[engine] lease query failed: {e:#}");
                tokio::select! {
                    _ = cancel.cancelled() => return,
                    _ = tokio::time::sleep(config.poll_interval) => {}
                }
            }
        }
    }
}

async fn shutdown_signal() -> Result<()> {
    #[cfg(unix)]
    {
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = sigterm.recv() => {}
        }
    }
    #[cfg(not(unix))]
    tokio::signal::ctrl_c().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = Arc::new(Config::from_env()?);

    let pool = PgPoolOptions::new()
        .max_connections(config.max_concurrent as u32 + 2)
        .connect(&config.database_url)
        .await
        .context("could not connect to the database")?;

    let recovered = db::release_leases(&pool, false).await?;
    if recovered > 0 {
        println!("[engine] returned {recovered} signal(s) from a previous run to pending");
    }

    let draining = Arc::new(AtomicBool::new(false));
    let health_app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .with_state(HealthState {
            draining: draining.clone(),
        });
    let health_addr = SocketAddr::from(([0, 0, 0, 0], config.health_port));
    let health_listener = tokio::net::TcpListener::bind(health_addr).await?;
    let health_server = tokio::spawn(async move {
        if let Err(e) = axum::serve(health_listener, health_app).await {
            eprintln!("[engine] health server error: {e}");
        }
    });

    let cancel = CancellationToken::new();
    let tracker = TaskTracker::new();
    let poller = tokio::spawn(poll_signals(
        pool.clone(),
        config.clone(),
        cancel.clone(),
        tracker.clone(),
    ));
    println!(
        "[engine] polling for signals, health on {health_addr}, up to {} at once",
        config.max_concurrent
    );

    shutdown_signal().await?;
    println!("[engine] shutting down");
    draining.store(true, Ordering::SeqCst);
    cancel.cancel();
    poller.await.ok();

    tracker.close();
    if tokio::time::timeout(DRAIN_TIMEOUT, tracker.wait())
        .await
        .is_err()
    {
        eprintln!("[engine] drain timed out, releasing remaining leases");
    }
    db::release_leases(&pool, false).await?;

    health_server.abort();
    Ok(())
}

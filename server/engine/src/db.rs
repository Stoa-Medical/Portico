use anyhow::{anyhow, Result};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::steps::StepType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    Run,
    Sync,
    Fyi,
}

impl SignalType {
    fn parse(s: &str) -> Result<Self> {
        match s {
            "run" => Ok(Self::Run),
            "sync" => Ok(Self::Sync),
            "fyi" => Ok(Self::Fyi),
            other => Err(anyhow!("unknown signal_type '{other}'")),
        }
    }
}

#[derive(Debug)]
pub struct LeasedSignal {
    pub id: Uuid,
    pub signal_type: SignalType,
    pub agent_id: Uuid,
    pub payload: Option<Value>,
}

#[derive(Debug)]
pub struct Step {
    pub id: Uuid,
    pub name: String,
    pub step_type: StepType,
    pub config: Value,
}

#[derive(Debug, Clone, Copy)]
pub enum Outcome {
    Completed,
    Failed,
}

impl Outcome {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug)]
pub struct RunRecord {
    pub outcome: Outcome,
    pub step_results: Vec<Value>,
    pub step_execution_ms: Vec<i64>,
    pub total_execution_ms: i64,
    pub response: Option<Value>,
    pub error: Option<String>,
}

/// Leases the oldest pending signal whose agent has nothing in flight, so each
/// agent's signals run one at a time in arrival order. Signals without an agent
/// are left pending.
pub async fn lease_next(pool: &PgPool, lease_seconds: f64) -> Result<Option<LeasedSignal>> {
    let row = sqlx::query(
        r#"
        UPDATE signals
        SET status = 'dispatched',
            leased_at = now(),
            lease_expires_at = now() + make_interval(secs => $1),
            updated_at = now()
        WHERE id = (
            SELECT s.id FROM signals s
            WHERE s.status = 'pending'
              AND s.agent_id IS NOT NULL
              AND NOT EXISTS (
                  SELECT 1 FROM signals d
                  WHERE d.agent_id = s.agent_id AND d.status = 'dispatched'
              )
            ORDER BY s.created_at
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING id, signal_type::text AS signal_type, agent_id, payload
        "#,
    )
    .bind(lease_seconds)
    .fetch_optional(pool)
    .await?;

    row.map(|r| {
        Ok(LeasedSignal {
            id: r.try_get("id")?,
            signal_type: SignalType::parse(r.try_get("signal_type")?)?,
            agent_id: r.try_get("agent_id")?,
            payload: r.try_get("payload")?,
        })
    })
    .transpose()
}

/// Returns dispatched signals to pending and closes their running sessions.
/// With `only_expired` false it releases every lease, which is correct at
/// startup and shutdown because a single engine owns all leases.
pub async fn release_leases(pool: &PgPool, only_expired: bool) -> Result<u64> {
    let released = sqlx::query(
        r#"
        WITH released AS (
            UPDATE signals
            SET status = 'pending', leased_at = NULL, lease_expires_at = NULL, updated_at = now()
            WHERE status = 'dispatched' AND (NOT $1 OR lease_expires_at < now())
            RETURNING id
        ), closed AS (
            UPDATE runtime_sessions
            SET status = 'cancelled', completed_at = now()
            WHERE status = 'running' AND signal_id IN (SELECT id FROM released)
        )
        SELECT count(*) AS n FROM released
        "#,
    )
    .bind(only_expired)
    .fetch_one(pool)
    .await?;
    Ok(released.try_get::<i64, _>("n")? as u64)
}

pub async fn load_steps(pool: &PgPool, agent_id: Uuid) -> Result<Vec<Step>> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, step_type::text AS step_type, config
        FROM steps
        WHERE agent_id = $1
        ORDER BY step_order, created_at
        "#,
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|r| {
            Ok(Step {
                id: r.try_get("id")?,
                name: r.try_get("name")?,
                step_type: StepType::parse(r.try_get("step_type")?)?,
                config: r
                    .try_get::<Option<Value>, _>("config")?
                    .unwrap_or(Value::Null),
            })
        })
        .collect()
}

pub async fn start_session(pool: &PgPool, signal: &LeasedSignal) -> Result<Uuid> {
    let row = sqlx::query(
        r#"
        INSERT INTO runtime_sessions (agent_id, signal_id, status)
        VALUES ($1, $2, 'running')
        RETURNING id
        "#,
    )
    .bind(signal.agent_id)
    .bind(signal.id)
    .fetch_one(pool)
    .await?;
    Ok(row.try_get("id")?)
}

/// Writes the session result and settles the signal in one transaction. The
/// signal update is guarded on `dispatched` so a lease that was already
/// released is not overwritten.
pub async fn finish_run(
    pool: &PgPool,
    signal_id: Uuid,
    session_id: Uuid,
    record: &RunRecord,
) -> Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        UPDATE runtime_sessions
        SET status = $2::running_status,
            step_results = $3,
            step_execution_ms = $4,
            total_execution_ms = $5,
            completed_at = now()
        WHERE id = $1
        "#,
    )
    .bind(session_id)
    .bind(record.outcome.as_str())
    .bind(Value::from(record.step_results.clone()))
    .bind(Value::from(record.step_execution_ms.clone()))
    .bind(i32::try_from(record.total_execution_ms).unwrap_or(i32::MAX))
    .execute(&mut *tx)
    .await?;

    settle_signal(
        &mut tx,
        signal_id,
        record.outcome,
        Some(session_id),
        record.response.as_ref(),
        record.error.as_deref(),
    )
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Settles a signal that needs no step run (`sync` and `fyi`).
pub async fn acknowledge(pool: &PgPool, signal_id: Uuid) -> Result<()> {
    let mut tx = pool.begin().await?;
    settle_signal(&mut tx, signal_id, Outcome::Completed, None, None, None).await?;
    tx.commit().await?;
    Ok(())
}

async fn settle_signal(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    signal_id: Uuid,
    outcome: Outcome,
    session_id: Option<Uuid>,
    response: Option<&Value>,
    error: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE signals
        SET status = $2::signal_status,
            runtime_session_id = $3,
            response_data = $4,
            error_message = $5,
            leased_at = NULL,
            lease_expires_at = NULL,
            updated_at = now()
        WHERE id = $1 AND status = 'dispatched'
        "#,
    )
    .bind(signal_id)
    .bind(outcome.as_str())
    .bind(session_id)
    .bind(response)
    .bind(error)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

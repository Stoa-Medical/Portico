use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::db::{self, LeasedSignal, Outcome, RunRecord, SignalType, Step};
use crate::steps;

/// Runs one leased signal to a settled state. Errors here are database
/// failures; the signal stays dispatched and is reclaimed when its lease
/// expires.
pub async fn process(pool: &PgPool, signal: LeasedSignal, budget: Duration) -> Result<()> {
    if signal.signal_type != SignalType::Run {
        return db::acknowledge(pool, signal.id).await;
    }

    let session_id = db::start_session(pool, &signal).await?;
    let started = Instant::now();

    let record = match db::load_steps(pool, signal.agent_id).await {
        Err(e) => failed(started, format!("could not load steps: {e:#}")),
        Ok(agent_steps) if agent_steps.is_empty() => {
            failed(started, "agent has no steps".to_string())
        }
        Ok(agent_steps) => {
            let input = signal.payload.clone().unwrap_or(Value::Null);
            match tokio::time::timeout(budget, run_steps(&agent_steps, input)).await {
                Ok(record) => record,
                Err(_) => failed(
                    started,
                    format!("run exceeded its {}s budget", budget.as_secs()),
                ),
            }
        }
    };

    db::finish_run(pool, signal.id, session_id, &record).await
}

fn failed(started: Instant, error: String) -> RunRecord {
    RunRecord {
        outcome: Outcome::Failed,
        step_results: Vec::new(),
        step_execution_ms: Vec::new(),
        total_execution_ms: started.elapsed().as_millis() as i64,
        response: None,
        error: Some(error),
    }
}

/// Feeds each step's output to the next and stops at the first failure,
/// keeping the results of the steps that ran.
async fn run_steps(agent_steps: &[Step], input: Value) -> RunRecord {
    let started = Instant::now();
    let mut step_results = Vec::with_capacity(agent_steps.len());
    let mut step_execution_ms = Vec::with_capacity(agent_steps.len());
    let mut current = input;
    let mut error = None;

    for step in agent_steps {
        let step_started = Instant::now();
        let result = steps::execute_step(step.step_type, &step.config, current.clone()).await;
        let ms = step_started.elapsed().as_millis() as i64;
        step_execution_ms.push(ms);

        match result {
            Ok(output) => {
                step_results.push(json!({
                    "step_id": step.id.to_string(),
                    "name": step.name,
                    "step_type": step.step_type.as_str(),
                    "status": "completed",
                    "output": output,
                }));
                current = output;
            }
            Err(e) => {
                let message = format!("{e:#}");
                step_results.push(json!({
                    "step_id": step.id.to_string(),
                    "name": step.name,
                    "step_type": step.step_type.as_str(),
                    "status": "failed",
                    "error": message,
                }));
                error = Some(format!("step '{}' failed: {message}", step.name));
                break;
            }
        }
    }

    let outcome = if error.is_some() {
        Outcome::Failed
    } else {
        Outcome::Completed
    };

    RunRecord {
        outcome,
        step_results,
        step_execution_ms,
        total_execution_ms: started.elapsed().as_millis() as i64,
        response: error.is_none().then_some(current),
        error,
    }
}

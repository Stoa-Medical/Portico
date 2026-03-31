use super::types::RuntimeSession;
use crate::RunningStatus;
use anyhow::{anyhow, Result};
use serde_json::Value;

impl RuntimeSession {
    /// Step execution has been moved to the engine's step dispatch system.
    /// This stub remains for API compatibility and returns an error directing
    /// callers to use the engine dispatch instead.
    pub async fn unified_start(&mut self) -> Result<Value> {
        self.status = RunningStatus::Cancelled;
        Err(anyhow!(
            "Step execution moved to engine step dispatch. Use the engine to execute sessions."
        ))
    }

    /// Start the session. Step execution has been moved to engine step dispatch.
    pub async fn start(&mut self) -> Result<Value> {
        self.unified_start().await
    }
}

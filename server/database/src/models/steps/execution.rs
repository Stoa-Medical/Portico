use super::types::Step;
use anyhow::{anyhow, Result};
use serde_json::Value;

impl Step {
    /// Step execution has been moved to the engine's step dispatch system.
    /// This stub remains for API compatibility.
    pub async fn run(
        &self,
        _source_data: Value,
        step_idx: usize,
    ) -> Result<Value> {
        Err(anyhow!(
            "Step execution moved to engine step dispatch (step_idx={}, type={})",
            step_idx,
            self.step_type.as_str()
        ))
    }
}

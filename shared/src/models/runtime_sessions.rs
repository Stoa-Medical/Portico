use crate::{IdFields, RunningStatus, TimestampFields};
use crate::Step;
use anyhow::{anyhow, Result};
use serde_json::Value;
use sqlx::Row;

#[derive(Debug)]
pub struct RuntimeSession {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
    pub steps: Vec<Step>,
    pub status: RunningStatus,
    pub source_data: Value,
    pub last_step_idx: Option<i32>,
    pub last_successful_result: Option<Value>,
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for RuntimeSession {

    // Expect a SQL query like:
    // ```sql
    // SELECT
    // rs.*,
    // COALESCE(
    //     (
    //         SELECT json_agg(json_build_object(
    //             'id', s.id,
    //             'global_uuid', s.global_uuid,
    //             'created_timestamp', s.created_timestamp,
    //             'last_updated_timestamp', s.last_updated_timestamp,
    //             'agent_id', s.agent_id,
    //             'name', s.name,
    //             'description', s.description,
    //             'step_type', s.step_type,
    //             'step_content', s.step_content,
    //             'success_count', s.success_count,
    //             'run_count', s.run_count
    //         ))
    //         FROM steps s
    //         WHERE s.runtime_session_id = rs.id
    //         ORDER BY s.sequence_number
    //     ),
    //     '[]'::json
    // ) as steps
    // FROM runtime_sessions rs
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        // Get the steps JSON array from the row
        let steps_json: Value = row.try_get("steps")?;

        // Convert the JSON array into Vec<Step> using the shared function
        let steps = Step::from_json_array(&steps_json);

        Ok(Self {
            identifiers: IdFields {
                local_id: row.try_get("id")?,
                global_uuid: row.try_get("global_uuid")?,
            },
            timestamps: TimestampFields {
                created: row.try_get("created_timestamp")?,
                updated: row.try_get("last_updated_timestamp")?,
            },
            steps,
            status: row.try_get("runtime_session_status")?,
            source_data: row.try_get("initial_data")?,
            last_step_idx: Some(row.try_get("latest_step_idx")?),
            last_successful_result: row.try_get("latest_result")?,
        })
    }
}

impl RuntimeSession {
    pub fn new(
        source_data: Value,
        steps: Vec<Step>,
    ) -> Self {
        Self {
            identifiers: IdFields::new(),
            timestamps: TimestampFields::new(),
            steps,
            status: RunningStatus::Waiting,
            source_data,
            last_step_idx: None,
            last_successful_result: None,
        }
    }

    pub async fn start(&mut self) -> Result<Value> {
        // Set status to Running
        self.status = RunningStatus::Running;

        // Execute each step in order, passing the result of each step to the next
        let mut current_value = self.source_data.clone();

        // Track step execution
        for (idx, step) in self.steps.iter().enumerate() {
            // Update latest step index before execution
            self.last_step_idx = Some(idx as i32);

            match step.run(current_value, idx).await {
                Ok(value) => {
                    // Update current value for next step
                    current_value = value.clone();

                    // Store the intermediate result
                    self.last_successful_result = Some(value);
                },
                Err(e) => {
                    // Update status to cancelled
                    self.status = RunningStatus::Cancelled;
                    return Err(anyhow!("Step execution failed: {}", e));
                }
            }
        }

        // All steps completed successfully
        self.status = RunningStatus::Completed;

        // Store the final result and return it
        self.last_successful_result = Some(current_value.clone());
        Ok(current_value)
    }
}

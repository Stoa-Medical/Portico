use crate::Step;
use crate::{DatabaseItem, IdFields, RunningStatus, TimestampFields};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

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
    //             'created_at', s.created_at,
    //             'updated_at', s.updated_at,
    //             'agent_id', s.agent_id,
    //             'description', s.description,
    //             'step_type', s.step_type,
    //             'step_content', s.step_content
    //         ))
    //         FROM steps s
    //         WHERE s.runtime_session_id = rs.id
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
                global_uuid: row.try_get::<Uuid, _>("global_uuid")?.to_string(),
            },
            timestamps: TimestampFields {
                created: row.try_get("created_at")?,
                updated: row.try_get("updated_at")?,
            },
            steps,
            status: row.try_get("rts_status")?,
            source_data: row.try_get("initial_data")?,
            last_step_idx: Some(row.try_get("latest_step_idx")?),
            last_successful_result: row.try_get("latest_result")?,
        })
    }
}

impl RuntimeSession {
    pub fn new(source_data: Value, steps: Vec<Step>) -> Self {
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
                }
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

#[async_trait]
impl DatabaseItem for RuntimeSession {
    fn id(&self) -> &IdFields {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        // Check if a session with the same UUID already exists
        if crate::check_exists_by_uuid(pool, "runtime_sessions", &self.identifiers.global_uuid).await? {
            return Ok(());  // Session already exists, no need to create it again
        }

        // First create the session record
        let record = sqlx::query(
            r#"
            INSERT INTO runtime_sessions (
                global_uuid, rts_status, initial_data,
                latest_step_idx, latest_result, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(Uuid::parse_str(&self.identifiers.global_uuid)?)
        .bind(&self.status)
        .bind(&self.source_data)
        .bind(&self.last_step_idx)
        .bind(&self.last_successful_result)
        .bind(&self.timestamps.created)
        .bind(&self.timestamps.updated)
        .fetch_one(pool)
        .await?;

        let session_id: i64 = record.get("id");

        // Then create step records if any exist
        for step in self.steps.iter() {
            sqlx::query(
                r#"
                INSERT INTO steps (
                    global_uuid, runtime_session_id, sequence_number, description,
                    step_type, step_content, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(&step.identifiers.global_uuid)
            .bind(session_id)
            .bind(0)  // Default sequence_number (not used for ordering anymore)
            .bind(&step.description)
            .bind(&step.step_type)
            .bind(&step.step_content)
            .bind(&step.timestamps.created)
            .bind(&step.timestamps.updated)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        // Update the session record
        sqlx::query(
            r#"
            UPDATE runtime_sessions
            SET rts_status = $1,
                initial_data = $2,
                latest_step_idx = $3,
                latest_result = $4,
                updated_at = $5
            WHERE global_uuid = $6
            "#,
        )
        .bind(&self.status)
        .bind(&self.source_data)
        .bind(&self.last_step_idx)
        .bind(&self.last_successful_result)
        .bind(&self.timestamps.updated)
        .bind(Uuid::parse_str(&self.identifiers.global_uuid)?)
        .execute(pool)
        .await?;

        // Steps should be updated through their own DatabaseItem implementation
        // since they have their own identifiers and lifecycle

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        // First delete associated steps
        if let Some(id) = self.identifiers.local_id {
            sqlx::query("DELETE FROM steps WHERE runtime_session_id = $1")
                .bind(id)
                .execute(pool)
                .await?;
        }

        // Then delete the session
        sqlx::query("DELETE FROM runtime_sessions WHERE global_uuid = $1")
            .bind(Uuid::parse_str(&self.identifiers.global_uuid)?)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let query = format!(
            r#"
            SELECT
                rs.*,
                {}
            FROM runtime_sessions rs
            "#,
            crate::steps_json_agg_sql("rs", "runtime_session_id")
        );

        let rows = sqlx::query_as::<_, Self>(&query)
            .fetch_all(pool)
            .await?;

        Ok(rows)
    }

    async fn try_db_select_by_id(pool: &PgPool, id: &IdFields) -> Result<Option<Self>> {
        let result = if let Some(local_id) = id.local_id {
            let query = format!(
                r#"
                SELECT
                    rs.*,
                    {}
                FROM runtime_sessions rs
                WHERE rs.id = $1
                "#,
                crate::steps_json_agg_sql("rs", "runtime_session_id")
            );

            sqlx::query_as::<_, Self>(&query)
                .bind(local_id)
                .fetch_optional(pool)
                .await?
        } else {
            let query = format!(
                r#"
                SELECT
                    rs.*,
                    {}
                FROM runtime_sessions rs
                WHERE rs.global_uuid = $1
                "#,
                crate::steps_json_agg_sql("rs", "runtime_session_id")
            );

            sqlx::query_as::<_, Self>(&query)
                .bind(Uuid::parse_str(&id.global_uuid)?)
                .fetch_optional(pool)
                .await?
        };

        Ok(result)
    }
}

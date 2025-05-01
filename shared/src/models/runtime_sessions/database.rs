use super::types::RuntimeSession;
use crate::{DatabaseItem, IdFields, RunningStatus, Step, TimestampFields};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::types::BigDecimal;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct RuntimeSessionRow {
    id: i64,
    global_uuid: Uuid,
    rts_status: RunningStatus,
    initial_data: Value,
    latest_step_idx: Option<i32>,
    latest_result: Option<Value>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    step_execution_times: Option<Vec<f64>>,
    total_execution_time: Option<f64>,
    steps: Value, // JSON aggregation result
    requested_by_agent_id: Option<i32>,
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for RuntimeSession {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        // Get the steps JSON array from the row
        let steps_json: Value = row.try_get("steps")?;

        // Convert the JSON array into Vec<Step> using the shared function
        let steps = Step::from_json_array(&steps_json);

        // Get execution times as array of numeric values
        let step_execution_times = match row.try_get::<Option<Vec<f64>>, _>("step_execution_times")
        {
            Ok(Some(times)) => {
                times
                    .into_iter()
                    .map(|seconds| {
                        // Convert seconds to Duration
                        let secs = seconds.trunc() as u64;
                        let nanos = ((seconds.fract() * 1_000_000_000.0) as u32).min(999_999_999);
                        Duration::new(secs, nanos)
                    })
                    .collect()
            }
            _ => Vec::new(),
        };

        // Get total execution time if available (stored as seconds in numeric type)
        let total_execution_time = match row.try_get::<Option<f64>, _>("total_execution_time") {
            Ok(Some(seconds)) => {
                // Convert seconds to Duration
                let secs = seconds.trunc() as u64;
                let nanos = ((seconds.fract() * 1_000_000_000.0) as u32).min(999_999_999);
                Duration::new(secs, nanos)
            }
            _ => Duration::ZERO,
        };

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
            step_execution_times,
            total_execution_time,
            requested_by_agent_id: row.try_get("requested_by_agent_id")?,
        })
    }
}

#[async_trait]
impl DatabaseItem for RuntimeSession {
    type IdType = i64;

    fn id(&self) -> &IdFields<Self::IdType> {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        // Check if a session with the same UUID already exists
        if crate::check_exists_by_uuid(pool, "runtime_sessions", &self.identifiers.global_uuid)
            .await?
        {
            return Ok(()); // Session already exists, no need to create it again
        }

        // Convert execution times to BigDecimal array
        let step_times_secs: Vec<BigDecimal> = self
            .step_execution_times
            .iter()
            .map(|duration| BigDecimal::from_str(&duration.as_secs_f64().to_string()).unwrap())
            .collect();

        // Collect step IDs from the steps vector
        let step_ids: Vec<i32> = self
            .steps
            .iter()
            .filter_map(|step| step.identifiers.local_id)
            .collect();

        // Convert Duration to BigDecimal seconds
        let total_time_secs =
            BigDecimal::from_str(&self.total_execution_time.as_secs_f64().to_string()).unwrap();

        // Parse UUID once for all operations
        let parsed_uuid = Uuid::parse_str(&self.identifiers.global_uuid)?;

        // Create the session record using query! macro
        sqlx::query!(
            r#"
            INSERT INTO runtime_sessions (
                global_uuid, rts_status, initial_data,
                latest_step_idx, latest_result, created_at, updated_at,
                step_execution_times, step_ids, total_execution_time, requested_by_agent_id
            )
            VALUES ($1, $2::running_status, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            parsed_uuid,
            &self.status as &RunningStatus,
            &self.source_data,
            self.last_step_idx,
            self.last_successful_result.as_ref().unwrap_or(&Value::Null),
            &self.timestamps.created,
            &self.timestamps.updated,
            &step_times_secs as &[BigDecimal],
            &step_ids,
            total_time_secs,
            self.requested_by_agent_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        // Convert execution times to BigDecimal array
        let step_times_secs: Vec<BigDecimal> = self
            .step_execution_times
            .iter()
            .map(|duration| BigDecimal::from_str(&duration.as_secs_f64().to_string()).unwrap())
            .collect();

        // Collect step IDs from the steps vector
        let step_ids: Vec<i32> = self
            .steps
            .iter()
            .filter_map(|step| step.identifiers.local_id)
            .collect();

        // Convert Duration to BigDecimal seconds
        let total_time_secs =
            BigDecimal::from_str(&self.total_execution_time.as_secs_f64().to_string()).unwrap();

        // Parse UUID once
        let parsed_uuid = Uuid::parse_str(&self.identifiers.global_uuid)?;

        sqlx::query!(
            r#"
            UPDATE runtime_sessions
            SET rts_status = $1::running_status,
                initial_data = $2,
                latest_step_idx = $3,
                latest_result = $4,
                updated_at = $5,
                step_execution_times = $6,
                step_ids = $7,
                total_execution_time = $8,
                requested_by_agent_id = $9
            WHERE global_uuid = $10
            "#,
            &self.status as &RunningStatus,
            &self.source_data,
            self.last_step_idx,
            self.last_successful_result.as_ref().unwrap_or(&Value::Null),
            &self.timestamps.updated,
            &step_times_secs as &[BigDecimal],
            &step_ids,
            total_time_secs,
            self.requested_by_agent_id,
            parsed_uuid
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        // Delete the session record
        let parsed_uuid = Uuid::parse_str(&self.identifiers.global_uuid)?;

        sqlx::query!(
            "DELETE FROM runtime_sessions WHERE global_uuid = $1",
            parsed_uuid
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let steps_json_agg = crate::steps_json_agg_sql("rs", "agent_id");

        // Use query_as! with our RuntimeSessionRow struct
        let query = format!(
            r#"
            SELECT
                rs.id as "id!",
                rs.global_uuid as "global_uuid!",
                rs.rts_status as "rts_status!: RunningStatus",
                rs.initial_data as "initial_data!",
                rs.latest_step_idx,
                rs.latest_result,
                rs.created_at as "created_at!",
                rs.updated_at as "updated_at!",
                rs.step_execution_times,
                rs.step_ids,
                rs.total_execution_time,
                {} as "steps!: Value"
            FROM runtime_sessions rs
            "#,
            steps_json_agg
        );

        let rows = sqlx::query_as::<_, RuntimeSessionRow>(&query)
            .fetch_all(pool)
            .await?;

        // Convert RuntimeSessionRow to RuntimeSession
        let sessions = rows
            .into_iter()
            .map(|row| RuntimeSession {
                identifiers: IdFields {
                    local_id: Some(row.id),
                    global_uuid: row.global_uuid.to_string(),
                },
                timestamps: TimestampFields {
                    created: row.created_at,
                    updated: row.updated_at,
                },
                steps: Step::from_json_array(&row.steps),
                status: row.rts_status,
                source_data: row.initial_data,
                last_step_idx: row.latest_step_idx,
                last_successful_result: row.latest_result,
                step_execution_times: row
                    .step_execution_times
                    .unwrap_or_default()
                    .into_iter()
                    .map(|secs| {
                        let secs_int = secs.trunc() as u64;
                        let nanos = ((secs.fract() * 1_000_000_000.0) as u32).min(999_999_999);
                        Duration::new(secs_int, nanos)
                    })
                    .collect(),
                total_execution_time: row
                    .total_execution_time
                    .map(|secs| {
                        let secs_int = secs.trunc() as u64;
                        let nanos = ((secs.fract() * 1_000_000_000.0) as u32).min(999_999_999);
                        Duration::new(secs_int, nanos)
                    })
                    .unwrap_or_default(),
                requested_by_agent_id: row.requested_by_agent_id,
            })
            .collect();

        Ok(sessions)
    }

    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>> {
        let steps_json_agg = crate::steps_json_agg_sql("rs", "agent_id");

        let row = if let Some(local_id) = id.local_id {
            let query = format!(
                r#"
                SELECT
                    rs.id as "id!",
                    rs.global_uuid as "global_uuid!",
                    rs.rts_status as "rts_status!: RunningStatus",
                    rs.initial_data as "initial_data!",
                    rs.latest_step_idx,
                    rs.latest_result,
                    rs.created_at as "created_at!",
                    rs.updated_at as "updated_at!",
                    rs.step_execution_times,
                    rs.step_ids,
                    rs.total_execution_time,
                    {} as "steps!: Value"
                FROM runtime_sessions rs
                WHERE rs.id = $1
                "#,
                steps_json_agg
            );

            sqlx::query_as::<_, RuntimeSessionRow>(&query)
                .bind(local_id)
                .fetch_optional(pool)
                .await?
        } else {
            let parsed_uuid = Uuid::parse_str(&id.global_uuid)?;

            let query = format!(
                r#"
                SELECT
                    rs.id as "id!",
                    rs.global_uuid as "global_uuid!",
                    rs.rts_status as "rts_status!: RunningStatus",
                    rs.initial_data as "initial_data!",
                    rs.latest_step_idx,
                    rs.latest_result,
                    rs.created_at as "created_at!",
                    rs.updated_at as "updated_at!",
                    rs.step_execution_times,
                    rs.step_ids,
                    rs.total_execution_time,
                    {} as "steps!: Value"
                FROM runtime_sessions rs
                WHERE rs.global_uuid = $1
                "#,
                steps_json_agg
            );

            sqlx::query_as::<_, RuntimeSessionRow>(&query)
                .bind(parsed_uuid)
                .fetch_optional(pool)
                .await?
        };

        Ok(row.map(|row| RuntimeSession {
            identifiers: IdFields {
                local_id: Some(row.id),
                global_uuid: row.global_uuid.to_string(),
            },
            timestamps: TimestampFields {
                created: row.created_at,
                updated: row.updated_at,
            },
            steps: Step::from_json_array(&row.steps),
            status: row.rts_status,
            source_data: row.initial_data,
            last_step_idx: row.latest_step_idx,
            last_successful_result: row.latest_result,
            step_execution_times: row
                .step_execution_times
                .unwrap_or_default()
                .into_iter()
                .map(|secs| {
                    let secs_int = secs.trunc() as u64;
                    let nanos = ((secs.fract() * 1_000_000_000.0) as u32).min(999_999_999);
                    Duration::new(secs_int, nanos)
                })
                .collect(),
            total_execution_time: row
                .total_execution_time
                .map(|secs| {
                    let secs_int = secs.trunc() as u64;
                    let nanos = ((secs.fract() * 1_000_000_000.0) as u32).min(999_999_999);
                    Duration::new(secs_int, nanos)
                })
                .unwrap_or_default(),
            requested_by_agent_id: row.requested_by_agent_id,
        }))
    }
}

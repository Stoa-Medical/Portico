/// Lib module (access with `crate::`)
///   Enums + traits go here (stylistic choice)!
/// Tests
#[cfg(test)]
mod tests;

/// Module with different data models
pub mod models;
pub use models::{
    Agent, RuntimeSession, Signal, Step, StepType,
    DataMapping, Integration, AuditLogEntry,
    DeadLetterSignal, OutboxEvent, EngineState,
};

// ============ Custom Enums / Traits ============
// === Imports ===
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row};
use uuid::Uuid;

// === Shared Enum definitions ===
#[cfg_attr(feature = "strum", derive(EnumString, AsRefStr, Display))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "running_status", rename_all = "lowercase")]
#[cfg_attr(feature = "strum", strum(serialize_all = "lowercase"))]
pub enum RunningStatus {
    Waiting,
    Running,
    Completed,
    Cancelled,
}

// ============ Struct definitions =============

#[derive(Clone, Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct IdFields<I = i32>
where
    I: sqlx::Type<Postgres>
        + for<'r> sqlx::Decode<'r, Postgres>
        + Send
        + Sync
        + Clone
        + std::fmt::Debug
        + 'static,
{
    pub local_id: Option<I>,
    pub global_uuid: String,
}

// Type aliases for common use cases
pub type IdFields32 = IdFields<i32>;
pub type IdFields64 = IdFields<i64>;

impl<I> Default for IdFields<I>
where
    I: sqlx::Type<Postgres>
        + for<'r> sqlx::Decode<'r, Postgres>
        + Send
        + Sync
        + Clone
        + std::fmt::Debug
        + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<I> IdFields<I>
where
    I: sqlx::Type<Postgres>
        + for<'r> sqlx::Decode<'r, Postgres>
        + Send
        + Sync
        + Clone
        + std::fmt::Debug
        + 'static,
{
    pub fn new() -> Self {
        Self {
            local_id: None,
            global_uuid: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn with_values(local_id: Option<I>, global_uuid: String) -> Self {
        Self {
            local_id,
            global_uuid,
        }
    }
}

#[derive(Clone, Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct TimestampFields {
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
}

impl Default for TimestampFields {
    fn default() -> Self {
        Self::new()
    }
}

impl TimestampFields {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            created: now,
            updated: now,
        }
    }

    pub fn update(&mut self) {
        self.updated = chrono::Utc::now();
    }
}

// ============ Trait definitions =============

/// Item that is in the `public` schema (Portico-custom, not Supabase-predefined)
#[async_trait]
pub trait DatabaseItem {
    /// The integer type used for the local_id (defaults to i32)
    type IdType: sqlx::Type<Postgres>
        + for<'r> sqlx::Decode<'r, Postgres>
        + Send
        + Sync
        + Clone
        + std::fmt::Debug
        + 'static;

    fn id(&self) -> &IdFields<Self::IdType>;
    async fn try_db_create(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_update(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_delete(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>>
    where
        Self: Sized;
    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>>
    where
        Self: Sized;
}

pub trait JsonLike {
    fn to_json(&self) -> Value;
    /// Creates new object
    fn from_json(obj: Value) -> Result<Self>
    where
        Self: Sized;
}

// ============ Shared functions ============

/// Checks if a record with the given UUID already exists in the specified table
pub async fn check_exists_by_uuid(pool: &PgPool, table: &str, uuid: &str) -> Result<bool> {
    let uuid_parsed = Uuid::parse_str(uuid)?;
    let query = format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE global_uuid = $1)",
        table
    );
    sqlx::query_scalar::<_, bool>(&query)
        .bind(uuid_parsed)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Failed to check if record exists: {}", e))
}

/// Returns a SQL fragment for Step JSON aggregation that's used in several queries
pub fn steps_json_agg_sql(parent_table: &str, parent_id_column: &str) -> String {
    format!(
        r#"COALESCE(
            (
                SELECT json_agg(json_build_object(
                    'id', s.id,
                    'global_uuid', s.global_uuid,
                    'created_at', s.created_at,
                    'updated_at', s.updated_at,
                    'name', s.name,
                    'description', s.description,
                    'step_type', s.step_type,
                    'config', s.config,
                    'step_order', s.step_order
                ) ORDER BY s.step_order ASC NULLS LAST, s.id ASC)
                FROM steps s
                WHERE s.{} = {}.id
            ),
            '[]'::json
        ) as steps"#,
        parent_id_column, parent_table
    )
}

/// Returns a SQL fragment for the common Signal-Agent JOIN query
pub fn signal_with_agent_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT
            s.*,
            a.id as agent_id,
            a.global_uuid as agent_global_uuid,
            a.description as agent_description,
            a.agent_state as agent_state,
            a.created_at as agent_created_at,
            a.updated_at as agent_updated_at
        FROM signals s
        LEFT JOIN agents a ON s.agent_id = a.id
        {}
        "#,
        where_clause
    )
}


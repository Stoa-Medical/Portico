use super::types::EngineState;
use anyhow::Result;
use sqlx::{PgPool, Row};

impl EngineState {
    /// Upsert the engine state (always uses id = 1 as the singleton row)
    pub async fn upsert(&self, pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO engine_state (id, last_redis_stream_id, last_heartbeat, updated_at)
            VALUES (1, $1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET
                last_redis_stream_id = EXCLUDED.last_redis_stream_id,
                last_heartbeat = EXCLUDED.last_heartbeat,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(self.last_redis_stream_id.as_deref())
        .bind(&self.last_heartbeat)
        .bind(&self.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Select the singleton engine state row
    pub async fn select(pool: &PgPool) -> Result<Option<Self>> {
        let row_opt = sqlx::query(
            r#"
            SELECT id, last_redis_stream_id, last_heartbeat, updated_at
            FROM engine_state
            WHERE id = 1
            "#,
        )
        .fetch_optional(pool)
        .await?;

        Ok(row_opt.and_then(|row| {
            Some(EngineState {
                id: row.try_get("id").ok()?,
                last_redis_stream_id: row.try_get("last_redis_stream_id").unwrap_or(None),
                last_heartbeat: row.try_get("last_heartbeat").unwrap_or(None),
                updated_at: row.try_get("updated_at").ok()?,
            })
        }))
    }
}

use super::types::DeadLetterSignal;
use anyhow::Result;
use sqlx::{PgPool, Row};

impl DeadLetterSignal {
    /// Insert a new dead letter signal
    pub async fn insert(&self, pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO dead_letter_signals (
                original_signal_id, signal_type, initial_data,
                error_message, retry_count, last_retry_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(self.original_signal_id)
        .bind(&self.signal_type)
        .bind(&self.initial_data)
        .bind(&self.error_message)
        .bind(self.retry_count)
        .bind(&self.last_retry_at)
        .bind(&self.created_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Select all dead letter signals, ordered by most recent first
    pub async fn select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let rows = sqlx::query(
            r#"
            SELECT id, original_signal_id, signal_type, initial_data,
                   error_message, retry_count, last_retry_at, created_at
            FROM dead_letter_signals
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        let items = rows
            .iter()
            .filter_map(|row| {
                Some(DeadLetterSignal {
                    id: row.try_get("id").ok(),
                    original_signal_id: row.try_get("original_signal_id").ok()?,
                    signal_type: row.try_get("signal_type").ok()?,
                    initial_data: row.try_get("initial_data").unwrap_or(None),
                    error_message: row.try_get("error_message").ok()?,
                    retry_count: row.try_get("retry_count").unwrap_or(0),
                    last_retry_at: row.try_get("last_retry_at").unwrap_or(None),
                    created_at: row.try_get("created_at").ok()?,
                })
            })
            .collect();

        Ok(items)
    }

    /// Select a dead letter signal by ID
    pub async fn select_by_id(pool: &PgPool, id: i64) -> Result<Option<Self>> {
        let row_opt = sqlx::query(
            r#"
            SELECT id, original_signal_id, signal_type, initial_data,
                   error_message, retry_count, last_retry_at, created_at
            FROM dead_letter_signals
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row_opt.and_then(|row| {
            Some(DeadLetterSignal {
                id: row.try_get("id").ok(),
                original_signal_id: row.try_get("original_signal_id").ok()?,
                signal_type: row.try_get("signal_type").ok()?,
                initial_data: row.try_get("initial_data").unwrap_or(None),
                error_message: row.try_get("error_message").ok()?,
                retry_count: row.try_get("retry_count").unwrap_or(0),
                last_retry_at: row.try_get("last_retry_at").unwrap_or(None),
                created_at: row.try_get("created_at").ok()?,
            })
        }))
    }

    /// Update retry count and last_retry_at
    pub async fn record_retry(&mut self, pool: &PgPool) -> Result<()> {
        let id = self.id.ok_or_else(|| anyhow::anyhow!("Cannot update without an ID"))?;
        self.retry_count += 1;
        self.last_retry_at = Some(chrono::Utc::now());

        sqlx::query(
            r#"
            UPDATE dead_letter_signals
            SET retry_count = $1, last_retry_at = $2
            WHERE id = $3
            "#,
        )
        .bind(self.retry_count)
        .bind(&self.last_retry_at)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete a dead letter signal (e.g., after successful reprocessing)
    pub async fn delete(pool: &PgPool, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM dead_letter_signals WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

use super::types::OutboxEvent;
use anyhow::Result;
use sqlx::{PgPool, Row};

impl OutboxEvent {
    /// Insert a new outbox event
    pub async fn insert(&self, pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO outbox_events (
                event_type, aggregate_type, aggregate_id,
                payload, published, published_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&self.event_type)
        .bind(&self.aggregate_type)
        .bind(&self.aggregate_id)
        .bind(&self.payload)
        .bind(self.published)
        .bind(&self.published_at)
        .bind(&self.created_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Select all unpublished outbox events, ordered oldest first
    pub async fn select_unpublished(pool: &PgPool) -> Result<Vec<Self>> {
        let rows = sqlx::query(
            r#"
            SELECT id, event_type, aggregate_type, aggregate_id,
                   payload, published, published_at, created_at
            FROM outbox_events
            WHERE published = false
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        let items = rows
            .iter()
            .filter_map(|row| {
                Some(OutboxEvent {
                    id: row.try_get("id").ok(),
                    event_type: row.try_get("event_type").ok()?,
                    aggregate_type: row.try_get("aggregate_type").ok()?,
                    aggregate_id: row.try_get("aggregate_id").ok()?,
                    payload: row.try_get("payload").ok()?,
                    published: row.try_get("published").unwrap_or(false),
                    published_at: row.try_get("published_at").unwrap_or(None),
                    created_at: row.try_get("created_at").ok()?,
                })
            })
            .collect();

        Ok(items)
    }

    /// Select all outbox events
    pub async fn select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let rows = sqlx::query(
            r#"
            SELECT id, event_type, aggregate_type, aggregate_id,
                   payload, published, published_at, created_at
            FROM outbox_events
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        let items = rows
            .iter()
            .filter_map(|row| {
                Some(OutboxEvent {
                    id: row.try_get("id").ok(),
                    event_type: row.try_get("event_type").ok()?,
                    aggregate_type: row.try_get("aggregate_type").ok()?,
                    aggregate_id: row.try_get("aggregate_id").ok()?,
                    payload: row.try_get("payload").ok()?,
                    published: row.try_get("published").unwrap_or(false),
                    published_at: row.try_get("published_at").unwrap_or(None),
                    created_at: row.try_get("created_at").ok()?,
                })
            })
            .collect();

        Ok(items)
    }

    /// Mark an outbox event as published
    pub async fn mark_published(pool: &PgPool, id: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox_events
            SET published = true, published_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete old published events (cleanup)
    pub async fn delete_published_before(
        pool: &PgPool,
        before: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM outbox_events
            WHERE published = true AND published_at < $1
            "#,
        )
        .bind(before)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

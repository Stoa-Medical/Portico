use super::types::AuditLogEntry;
use anyhow::Result;
use sqlx::{PgPool, Row};

impl AuditLogEntry {
    /// Insert a new audit log entry (append-only)
    pub async fn insert(&self, pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_log (actor, action, resource_type, resource_id, details, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&self.actor)
        .bind(&self.action)
        .bind(&self.resource_type)
        .bind(self.resource_id.as_deref())
        .bind(&self.details)
        .bind(&self.created_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Select all audit log entries, ordered by most recent first
    pub async fn select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let rows = sqlx::query(
            r#"
            SELECT id, actor, action, resource_type, resource_id, details, created_at
            FROM audit_log
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        let entries = rows
            .iter()
            .filter_map(|row| {
                Some(AuditLogEntry {
                    id: row.try_get("id").ok(),
                    actor: row.try_get("actor").ok()?,
                    action: row.try_get("action").ok()?,
                    resource_type: row.try_get("resource_type").ok()?,
                    resource_id: row.try_get("resource_id").unwrap_or(None),
                    details: row.try_get("details").unwrap_or(None),
                    created_at: row.try_get("created_at").ok()?,
                })
            })
            .collect();

        Ok(entries)
    }
}

use super::types::{Agent, AgentState};
use crate::models::steps::Step;
use crate::{DatabaseItem, IdFields, JsonLike, TimestampFields};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::types::JsonValue;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use uuid::Uuid;

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Agent {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        let id: i32 = row.try_get("id")?;
        let global_uuid: uuid::Uuid = row.try_get("global_uuid")?;
        let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;
        let updated_at: chrono::DateTime<chrono::Utc> = row.try_get("updated_at")?;
        let description: String = row
            .try_get::<Option<String>, _>("description")?
            .unwrap_or_default();
        let agent_state: AgentState = row.try_get("agent_state")?;

        // Parse steps - each raw JSON will look like a `json_build_object` result
        let steps_json: Value = row.try_get("steps")?;
        let steps = Step::from_json_array(&steps_json);

        Ok(Self {
            identifiers: IdFields {
                local_id: Some(id),
                global_uuid: global_uuid.to_string(),
            },
            timestamps: TimestampFields {
                created: created_at,
                updated: updated_at,
            },
            description,
            agent_state: std::sync::Mutex::new(agent_state),
            steps,
        })
    }
}

impl JsonLike for Agent {
    fn to_json(&self) -> Value {
        serde_json::json!({
            "id": self.identifiers.local_id,
            "global_uuid": self.identifiers.global_uuid,
            "created_at": self.timestamps.created.format("%Y-%m-%d %H:%M:%S").to_string(),
            "updated_at": self.timestamps.updated.format("%Y-%m-%d %H:%M:%S").to_string(),
            "description": self.description,
            "agent_state": self.state(),
            "steps": self.steps.iter().map(|step| step.to_json()).collect::<Vec<Value>>(),
        })
    }

    fn from_json(obj: Value) -> Result<Self> {
        if let Some(obj) = obj.as_object() {
            Ok(Self {
                identifiers: IdFields {
                    local_id: obj.get("id").and_then(|v| v.as_i64()).map(|v| v as i32),
                    global_uuid: obj
                        .get("global_uuid")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                },
                timestamps: TimestampFields {
                    created: chrono::DateTime::parse_from_str(
                        &obj.get("created_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default(),
                        "%Y-%m-%d %H:%M:%S %z",
                    )
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                    updated: chrono::DateTime::parse_from_str(
                        &obj.get("updated_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default(),
                        "%Y-%m-%d %H:%M:%S %z",
                    )
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                },
                description: obj
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                agent_state: std::sync::Mutex::new(
                    obj.get("agent_state")
                        .and_then(|v| v.as_str())
                        .and_then(|s| AgentState::from_str(s).ok())
                        .unwrap_or_default(),
                ),
                steps: obj
                    .get("steps")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|step| Step::from_json(step.clone()).ok())
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        } else {
            Err(anyhow!("Expected JSON object"))
        }
    }
}

#[async_trait]
impl DatabaseItem for Agent {
    type IdType = i32;

    fn id(&self) -> &IdFields<Self::IdType> {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        // Check if an agent with the same UUID already exists
        if crate::check_exists_by_uuid(pool, "agents", &self.identifiers.global_uuid).await? {
            return Ok(()); // Agent already exists, no need to create it again
        }

        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let agent_state = self.state(); // Get the current state

        // Use query_scalar! for inserting the agent and returning the ID
        let agent_id = sqlx::query_scalar!(
            r#"
            INSERT INTO agents (
                global_uuid, description, agent_state, created_at, updated_at
            )
            VALUES ($1, $2, $3::agent_state, $4, $5)
            RETURNING id
            "#,
            uuid_parsed,
            &self.description,
            agent_state as AgentState,
            &self.timestamps.created,
            &self.timestamps.updated
        )
        .fetch_one(pool)
        .await?;

        // Then create step records if any exist
        for step in self.steps.iter() {
            let step_uuid = Uuid::parse_str(&step.identifiers.global_uuid)?;
            let step_type_str = step.step_type.as_str();

            sqlx::query!(
                r#"
                INSERT INTO steps (
                    global_uuid, agent_id, description,
                    step_type, step_content, created_at, updated_at
                )
                VALUES ($1, $2, $3, ($4::text)::step_type, $5, $6, $7)
                "#,
                step_uuid,
                agent_id,
                step.description.as_deref().unwrap_or(""),
                step_type_str,
                &step.step_content,
                &step.timestamps.created,
                &step.timestamps.updated
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let agent_state = self.state();

        sqlx::query!(
            r#"
            UPDATE agents
            SET description = $1,
                agent_state = $2::agent_state,
                updated_at = $3
            WHERE global_uuid = $4
            "#,
            &self.description,
            agent_state as AgentState,
            &self.timestamps.updated,
            uuid_parsed
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        if let Some(id) = self.identifiers.local_id {
            sqlx::query!("DELETE FROM steps WHERE agent_id = $1", id)
                .execute(pool)
                .await?;
        }

        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        sqlx::query!("DELETE FROM agents WHERE global_uuid = $1", uuid_parsed)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        struct AgentRow {
            id: i32,
            global_uuid: uuid::Uuid,
            description: Option<String>,
            agent_state: AgentState,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            steps: serde_json::Value,
        }

        let rows = sqlx::query_as!(
            AgentRow,
            r#"
            SELECT
                a.id, a.global_uuid, a.description,
                a.agent_state as "agent_state: _",
                a.created_at, a.updated_at,
                COALESCE(
                    (
                        SELECT json_agg(json_build_object(
                            'id', s.id,
                            'global_uuid', s.global_uuid,
                            'created_at', s.created_at,
                            'updated_at', s.updated_at,
                            'agent_id', s.agent_id,
                            'description', s.description,
                            'step_type', s.step_type::text,
                            'step_content', s.step_content
                        ))
                        FROM steps s
                        WHERE s.agent_id = a.id
                    ),
                    '[]'::json
                ) as "steps: JsonValue"
            FROM agents a
            "#
        )
        .fetch_all(pool)
        .await?;

        let agents = rows
            .into_iter()
            .map(|row| {
                let steps = Step::from_json_array(&row.steps);

                Agent {
                    identifiers: IdFields {
                        local_id: Some(row.id),
                        global_uuid: row.global_uuid.to_string(),
                    },
                    timestamps: TimestampFields {
                        created: row.created_at,
                        updated: row.updated_at,
                    },
                    description: row.description.unwrap_or_default(),
                    agent_state: std::sync::Mutex::new(row.agent_state),
                    steps,
                }
            })
            .collect();

        Ok(agents)
    }

    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>> {
        struct AgentRow {
            id: i32,
            global_uuid: uuid::Uuid,
            description: Option<String>,
            agent_state: AgentState,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            steps: serde_json::Value,
        }

        let row_opt = if let Some(local_id) = id.local_id {
            sqlx::query_as!(
                AgentRow,
                r#"
                SELECT
                    a.id, a.global_uuid, a.description,
                    a.agent_state as "agent_state: _",
                    a.created_at, a.updated_at,
                    COALESCE(
                        (
                            SELECT json_agg(json_build_object(
                                'id', s.id,
                                'global_uuid', s.global_uuid,
                                'created_at', s.created_at,
                                'updated_at', s.updated_at,
                                'agent_id', s.agent_id,
                                'description', s.description,
                                'step_type', s.step_type::text,
                                'step_content', s.step_content
                            ))
                            FROM steps s
                            WHERE s.agent_id = a.id
                        ),
                        '[]'::json
                    ) as "steps: JsonValue"
                FROM agents a
                WHERE a.id = $1
                "#,
                local_id
            )
            .fetch_optional(pool)
            .await?
        } else {
            let uuid_parsed = Uuid::parse_str(&id.global_uuid)?;
            sqlx::query_as!(
                AgentRow,
                r#"
                SELECT
                    a.id, a.global_uuid, a.description,
                    a.agent_state as "agent_state: _",
                    a.created_at, a.updated_at,
                    COALESCE(
                        (
                            SELECT json_agg(json_build_object(
                                'id', s.id,
                                'global_uuid', s.global_uuid,
                                'created_at', s.created_at,
                                'updated_at', s.updated_at,
                                'agent_id', s.agent_id,
                                'description', s.description,
                                'step_type', s.step_type::text,
                                'step_content', s.step_content
                            ))
                            FROM steps s
                            WHERE s.agent_id = a.id
                        ),
                        '[]'::json
                    ) as "steps: JsonValue"
                FROM agents a
                WHERE a.global_uuid = $1
                "#,
                uuid_parsed
            )
            .fetch_optional(pool)
            .await?
        };

        Ok(row_opt.map(|row| {
            let steps = Step::from_json_array(&row.steps);

            Agent {
                identifiers: IdFields {
                    local_id: Some(row.id),
                    global_uuid: row.global_uuid.to_string(),
                },
                timestamps: TimestampFields {
                    created: row.created_at,
                    updated: row.updated_at,
                },
                description: row.description.unwrap_or_default(),
                agent_state: std::sync::Mutex::new(row.agent_state),
                steps,
            }
        }))
    }
}

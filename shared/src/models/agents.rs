/// An Agent represents a component that listens for and reacts to Signals in the system.
/// Agents are responsible for monitoring specific Signal types and acting on them
/// NOTE: Agents are created in the UI, and Supabase is the source-of-truth for their state.
use crate::models::{runtime_sessions::RuntimeSession, steps::Step};
use crate::{DatabaseItem, IdFields, JsonLike, PythonRuntime, TimestampFields};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::JsonValue;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
    pub description: String,
    pub agent_state: Mutex<AgentState>, // Make public for direct construction in other modules
    pub steps: Vec<Step>,
}

/// Different states for Agent to be in. State diagram:
/// ```plain
///          (start)    ┌──────────┐
///  Inactive ───────► Stable ──┐  │
///      ▲              ▲       │  │
///      │              │ (err) │  │
///      │          Unstable ◄──┘  │
///      │   (stop)     │          │
///      └──────────────┘◄─────────┘
/// ```
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default, sqlx::Type)]
#[sqlx(type_name = "agent_state", rename_all = "lowercase")]
pub enum AgentState {
    #[default]
    Inactive,
    Stable,
    Unstable,
}

impl AgentState {
    pub fn as_str(&self) -> &str {
        match self {
            AgentState::Inactive => "inactive",
            AgentState::Stable => "stable",
            AgentState::Unstable => "unstable",
        }
    }
}

impl std::fmt::Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AgentState::Inactive => "inactive",
            AgentState::Stable => "stable",
            AgentState::Unstable => "unstable",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for AgentState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "inactive" => Ok(AgentState::Inactive),
            "stable" => Ok(AgentState::Stable),
            "unstable" => Ok(AgentState::Unstable),
            _ => Err(format!("Unknown agent state: {}", s)),
        }
    }
}

impl Agent {
    // Add getter and setter for agent_state
    pub fn state(&self) -> AgentState {
        let guard = self.agent_state.lock().unwrap();
        guard.clone()
    }

    pub fn set_state(&self, new_state: AgentState) {
        let mut guard = self.agent_state.lock().unwrap();
        *guard = new_state;
    }

    pub fn new(
        identifiers: IdFields,
        timestamps: TimestampFields,
        description: String,
        steps: Vec<Step>,
    ) -> Self {
        // Start all agents in an inactive state
        Self {
            identifiers,
            timestamps,
            description,
            agent_state: Mutex::new(AgentState::Inactive),
            steps,
        }
    }

    pub fn start(&self) -> Result<()> {
        let current_state = self.state();
        match current_state {
            AgentState::Inactive => {
                // Set new state to Stable
                self.set_state(AgentState::Stable);
                Ok(())
            }
            _ => Err(anyhow!("Can only start from Inactive state")),
        }
    }

    /// Create a Python runtime for this agent
    pub fn create_python_runtime(&self) -> Result<PythonRuntime> {
        let mut runtime = PythonRuntime::new(&self.identifiers.global_uuid)?;

        // Add all Python steps
        for step in &self.steps {
            if step.is_python_step() {
                runtime.add_step(step)?;
            }
        }

        Ok(runtime)
    }

    /// Process data with this agent using an immutable reference
    pub async fn run(&self, source: Value) -> Result<RuntimeSession> {
        // Check if state is Inactive. If so, return error
        if self.state() == AgentState::Inactive {
            return Err(anyhow!("Cannot run agent in Inactive state"));
        }

        // Create a Python runtime for this agent
        let runtime = self.create_python_runtime()?;

        // Create a new RuntimeSession with the agent's steps
        let mut session = RuntimeSession::new(source, self.steps.clone());

        // Start the RuntimeSession with the Python runtime
        let result = session.start_with_runtime(&runtime).await;

        // If there was an error, propagate it
        if let Err(e) = result {
            return Err(e);
        }

        // Return final session
        Ok(session)
    }

    pub fn stop(&self) -> Result<()> {
        // Set to inactive
        let current_state = self.state();
        match current_state {
            AgentState::Stable | AgentState::Unstable => {
                self.set_state(AgentState::Inactive);
                Ok(())
            }
            _ => Err(anyhow!("Can only stop from a running state")),
        }
    }
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Agent {
    // Expect a SQL query like:
    // ```sql
    // SELECT
    // a.*,
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
    //         WHERE s.agent_id = a.id
    //     ),
    //     '[]'::json
    // ) as steps
    // FROM agents a
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
            agent_state: Mutex::new(agent_state),
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
                agent_state: Mutex::new(
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
        // Note: Direct binding of AgentState requires it to derive sqlx::Type
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
            agent_state as AgentState, // Cast enum for type checking
            &self.timestamps.created,
            &self.timestamps.updated
        )
        .fetch_one(pool)
        .await?;

        // Then create step records if any exist
        for step in self.steps.iter() {
            let step_uuid = Uuid::parse_str(&step.identifiers.global_uuid)?;
            let step_type_str = step.step_type.as_str(); // Assuming StepType has as_str()

            // Use query! for inserting steps
            // Note: Binding step_type requires casting if it doesn't derive sqlx::Type directly
            sqlx::query!(
                r#"
                INSERT INTO steps (
                    global_uuid, agent_id, description,
                    step_type, step_content, created_at, updated_at
                )
                VALUES ($1, $2, $3, ($4::text)::step_type, $5, $6, $7)
                "#,
                step_uuid,
                agent_id, // Use the returned agent_id
                step.description.as_deref().unwrap_or(""),
                step_type_str, // Bind as string, cast in SQL
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
        // Update the agent record
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let agent_state = self.state(); // Get the current state

        // Use query! for updating the agent
        sqlx::query!(
            r#"
            UPDATE agents
            SET description = $1,
                agent_state = $2::agent_state,
                updated_at = $3
            WHERE global_uuid = $4
            "#,
            &self.description,
            agent_state as AgentState, // Cast enum for type checking
            &self.timestamps.updated,
            uuid_parsed
        )
        .execute(pool)
        .await?;

        // Steps should be updated through their own DatabaseItem implementation
        // since they have their own identifiers and lifecycle

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        // This already uses query! macros, no changes needed here.

        // First delete associated steps
        if let Some(id) = self.identifiers.local_id {
            sqlx::query!("DELETE FROM steps WHERE agent_id = $1", id)
                .execute(pool)
                .await?;
        }

        // Then delete the agent
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        sqlx::query!("DELETE FROM agents WHERE global_uuid = $1", uuid_parsed)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        // Define struct compatible with query_as! output
        // Ensure AgentState derives sqlx::Type
        struct AgentRow {
            id: i32,
            global_uuid: uuid::Uuid,
            description: Option<String>,
            agent_state: AgentState, // Direct enum type
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            steps: serde_json::Value, // Keep as JSON for aggregation
        }

        // Use query_as! with inlined steps aggregation
        let rows = sqlx::query_as!(
            AgentRow,
            r#"
            SELECT
                a.id, a.global_uuid, a.description,
                a.agent_state as "agent_state: _", -- Tell sqlx to use AgentState type
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
                            'step_type', s.step_type::text, -- Cast enum to text for JSON
                            'step_content', s.step_content
                        ))
                        FROM steps s
                        WHERE s.agent_id = a.id
                    ),
                    '[]'::json
                ) as "steps: JsonValue" -- Tell sqlx to use Value type
            FROM agents a
            "#
        )
        .fetch_all(pool)
        .await?;

        // Convert rows to Agent objects
        let agents = rows
            .into_iter()
            .map(|row| {
                let steps = Step::from_json_array(&row.steps); // Use existing parser

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
                    agent_state: Mutex::new(row.agent_state), // Directly use the enum
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
        // Define struct compatible with query_as! output
        struct AgentRow {
            id: i32,
            global_uuid: uuid::Uuid,
            description: Option<String>,
            agent_state: AgentState, // Direct enum type
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            steps: serde_json::Value, // Keep as JSON for aggregation
        }

        let row_opt = if let Some(local_id) = id.local_id {
            // Use query_as! by local ID with inlined steps aggregation
            sqlx::query_as!(
                AgentRow,
                r#"
                SELECT
                    a.id, a.global_uuid, a.description,
                    a.agent_state as "agent_state: _", -- Use AgentState type
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
                                'step_type', s.step_type::text, -- Cast enum to text for JSON
                                'step_content', s.step_content
                            ))
                            FROM steps s
                            WHERE s.agent_id = a.id
                        ),
                        '[]'::json
                    ) as "steps: JsonValue" -- Use Value type
                FROM agents a
                WHERE a.id = $1
                "#,
                local_id
            )
            .fetch_optional(pool)
            .await?
        } else {
            // Use query_as! by global UUID with inlined steps aggregation
            let uuid_parsed = Uuid::parse_str(&id.global_uuid)?;
            sqlx::query_as!(
                AgentRow,
                r#"
                SELECT
                    a.id, a.global_uuid, a.description,
                    a.agent_state as "agent_state: _", -- Use AgentState type
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
                                'step_type', s.step_type::text, -- Cast enum to text for JSON
                                'step_content', s.step_content
                            ))
                            FROM steps s
                            WHERE s.agent_id = a.id
                        ),
                        '[]'::json
                    ) as "steps: JsonValue" -- Use Value type
                FROM agents a
                WHERE a.global_uuid = $1
                "#,
                uuid_parsed
            )
            .fetch_optional(pool)
            .await?
        };

        // Convert row to Agent if found
        Ok(row_opt.map(|row| {
            let steps = Step::from_json_array(&row.steps); // Use existing parser

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
                agent_state: Mutex::new(row.agent_state), // Directly use the enum
                steps,
            }
        }))
    }
}

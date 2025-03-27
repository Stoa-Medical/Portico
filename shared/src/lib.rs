/// Lib module (access with `crate::`)
///   Enums + traits go here (stylistic choice)!

/// Module with different data models
pub mod models;
pub use models::{Agent, RuntimeSession, Signal, Step};

// ============ Custom Enums / Traits ============
// === Imports ===
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use std::env;
#[cfg(feature = "sqlx-postgres")]
use sqlx::postgres::PgPool;
use std::ffi::CString;
use pyo3::prelude::*;
use pyo3::types::PyDict;

// === Shared Enum definitions ===
#[derive(Debug, PartialEq)]
pub enum RunningStatus {
    Waiting,
    Running,
    Completed,
    Cancelled,
}

// ============ Struct definitions =============

#[derive(Clone)]
pub struct IdFields {
    pub local_id: Option<u64>,
    pub global_uuid: String,
}

impl Default for IdFields {
    fn default() -> Self {
        Self::new()
    }
}

impl IdFields {
    // TODO: Have some way to increment correctly
    //    Primarily matters for `RuntimeSession` which is created here
    //    (everything else is created in the UI, and Supabase is the source-of-truth)
    pub fn new() -> Self {
        Self {
            local_id: None,
            global_uuid: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn with_values(local_id: Option<u64>, global_uuid: String) -> Self {
        Self { local_id, global_uuid }
    }
}

#[derive(Clone)]
pub struct TimestampFields {
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}

impl Default for TimestampFields {
    fn default() -> Self {
        Self::new()
    }
}

impl TimestampFields {
    pub fn new() -> Self {
        let now = chrono::Local::now().naive_utc();
        Self {
            created: now,
            updated: now,
        }
    }

    pub fn update(&mut self) {
        self.updated = chrono::Local::now().naive_utc();
    }
}

// ============ Trait definitions =============

/// Item that is in the `public` schema (Portico-custom, not Supabase-predefined)
#[allow(async_fn_in_trait)]
#[cfg(feature = "sqlx-postgres")]
pub trait DatabaseItem: Sized {
    async fn try_db_create(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_update(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_delete(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>>;
}

// ============ Shared functions ============

// TODO: Add various supported TogetherAI models
pub async fn call_llm(prompt: &str, context: Value) -> Result<String> {
    let api_key = env::var("LLM_API_KEY").unwrap();
    let api_endpoint = env::var("LLM_API_ENDPOINT").unwrap();

    let request = serde_json::json!({
        "model": "meta-llama/Llama-3.3-70B-Instruct-Turbo",
        "prompt": format!("{} | Context: {}", prompt, context),
        "max_tokens": 1000,
        "temperature": 0.7
    });

    let response: Value = Client::new()
        .post(api_endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    response["choices"][0]["message"]["content"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("No completion found"))
}

/// Executes provided python code
// TODO: Refactor this to enforce that the code is a pure function, and the function should just be called
pub fn exec_python(source: Value, the_code: &str) -> Result<Value> {
    // Preps python interpreter (needs to run at least once, and repeat calls are negligible)
    pyo3::prepare_freethreaded_python();
    // Run code with independent context
    Python::with_gil(|py| {
        // Have clean state at each start
        let locals = PyDict::new(py);
        // Convert serde_json::Value to PyObject
        let incoming_data = serde_json::to_string(&source)?;
        locals.set_item("source", incoming_data)?;

        // Convert String to CString correctly
        let code_as_cstr = CString::new(the_code.as_bytes())?;
        py.run(code_as_cstr.as_c_str(), None, Some(&locals))?;

        // Get result and convert back to serde_json::Value if it exists
        match locals.get_item("result") {
            Ok(Some(res)) => {
                let res_str = res.to_string();
                let json_value: Value = serde_json::from_str(&res_str)?;
                Ok(json_value)
            }
            Ok(None) => Err(anyhow!("Runtime error: unable to find return value (`result`)")),
            Err(err) => Err(anyhow!("Python error: {}", err)),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;
    use uuid::Uuid;

    mod test_utils {
        use super::*;

        #[test]
        fn test_exec_python() {
            // Test a simple Python function that adds numbers
            let source = json!({"a": 5, "b": 7});
            let python_code = r#"
result = {"sum": source["a"] + source["b"]}
"#;

            let result = exec_python(source, python_code);
            assert!(result.is_ok(), "Python execution should succeed");

            if let Ok(value) = result {
                assert_eq!(value, json!({"sum": 12}));
            }
        }

        // Note: call_llm would need mocking for testing
    }

    mod test_agents {
        use super::*;

        #[test]
        fn test_new_agent() {
            let id_fields = IdFields::new();
            let timestamps = TimestampFields::new();
            let description = "Test Agent".to_string();
            let accepted_rate = 0.8;
            let steps = vec![];

            let mut agent = Agent::new(
                id_fields,
                timestamps,
                description.clone(),
                accepted_rate,
                steps,
                0,
                0
            );

            // Test initial state - agent should start inactive
            // Since we can't access private fields directly, we'll test behavior
            assert!(agent.start().is_ok(), "New agent should be able to start");

            // Create a new agent to test the start fails when already started
            let mut another_agent = Agent::new(
                IdFields::new(),
                TimestampFields::new(),
                description,
                accepted_rate,
                vec![],
                0,
                0
            );

            // Start once
            assert!(another_agent.start().is_ok());
            // Second start should fail
            assert!(another_agent.start().is_err(), "Cannot start an already started agent");
        }

        #[test]
        fn test_agent_state_transitions() {
            let id_fields = IdFields::new();
            let timestamps = TimestampFields::new();
            let description = "Test Agent".to_string();
            let accepted_rate = 0.8;
            let steps = vec![];

            let mut agent = Agent::new(
                id_fields,
                timestamps,
                description,
                accepted_rate,
                steps,
                0,
                0
            );

            // Test state transitions via available methods

            // Start the agent - should transition to a running state
            assert!(agent.start().is_ok(), "Agent should start successfully");

            // Stop the agent - should transition back to Inactive
            assert!(agent.stop().is_ok(), "Agent should stop successfully");

            // Stopping an inactive agent should fail
            assert!(agent.stop().is_err(), "Stopping inactive agent should fail");
        }

        #[test]
        fn test_completion_rate() {
            let id_fields = IdFields::new();
            let timestamps = TimestampFields::new();
            let description = "Test Agent".to_string();
            let accepted_rate = 0.8;
            let steps = vec![];

            // Create agent with completion history
            let mut agent = Agent::new(
                id_fields,
                timestamps,
                description,
                accepted_rate,
                steps,
                4,  // completed
                5   // total runs
            );

            // Since the completion rate is private, we can test related behaviors
            // rather than the direct value. For instance, if completion rate is good:
            assert!(agent.start().is_ok());
            // If we had a method to get completion rate, we'd use that instead
        }
    }

    mod test_steps {
        use super::*;

        fn create_test_step(step_type: models::steps::StepType) -> Step {
            let id_fields = IdFields::new();
            let agent_uuid = Uuid::new_v4();
            let content = match step_type {
                models::steps::StepType::Python => "source['value'] += 10\nresult = source".to_string(),
                models::steps::StepType::Prompt => "Add 10 to the value in the data".to_string()
            };

            Step::new(
                id_fields,
                agent_uuid,
                step_type,
                content,
                "Test Step".to_string(),
                Some("A test step".to_string()),
                0,
                0
            )
        }

        #[test]
        fn test_step_creation() {
            let step = create_test_step(models::steps::StepType::Python);

            assert_eq!(step.get_run_count(), 0);
            assert_eq!(step.get_success_count(), 0);
        }

        #[test]
        fn test_step_counters() {
            let step = create_test_step(models::steps::StepType::Python);

            // Initial counts should be 0
            assert_eq!(step.get_run_count(), 0);
            assert_eq!(step.get_success_count(), 0);

            // We can't directly modify private fields, so we'd need to mock
            // or actually run a step which would increment these counters
            // In real tests you might use test doubles/mocks
        }

        #[test]
        fn test_step_type_behavior() {
            let _python_step = create_test_step(models::steps::StepType::Python);
            let _prompt_step = create_test_step(models::steps::StepType::Prompt);

            // In real tests, we'd test different behavior based on step type
            // For example, the Python step should execute Python code
            // and the Prompt step should call the LLM
            // Here we'd mock those dependencies
        }
    }

    mod test_signals {
        use super::*;

        fn create_test_signal() -> Signal {
            let id_fields = IdFields::new();
            let user_uuid = Uuid::new_v4().to_string();

            // Create a simple agent for the signal
            let agent_id = IdFields::new();
            let timestamps = TimestampFields::new();
            let agent = Agent::new(
                agent_id,
                timestamps,
                "Test Agent".to_string(),
                0.8,
                vec![],
                0,
                0
            );

            let initial_data = Some(json!({"value": 5}));

            Signal::new(
                id_fields,
                user_uuid,
                agent,
                "Test Signal".to_string(),
                initial_data
            )
        }

        #[test]
        fn test_signal_creation() {
            let _signal = create_test_signal();
            // Test signal creation behavior
            // In a more complete test, we would test actual processing
        }

        #[test]
        fn test_signal_without_data() {
            let id_fields = IdFields::new();
            let user_uuid = Uuid::new_v4().to_string();

            // Create a simple agent
            let agent_id = IdFields::new();
            let timestamps = TimestampFields::new();
            let agent = Agent::new(
                agent_id,
                timestamps,
                "Test Agent".to_string(),
                0.8,
                vec![],
                0,
                0
            );

            // Create signal with no initial data
            let mut signal = Signal::new(
                id_fields,
                user_uuid,
                agent,
                "Test Signal".to_string(),
                None  // No data
            );

            // Processing should fail without data
            let process_result = tokio_test::block_on(signal.process());
            assert!(process_result.is_err(), "Process should fail without data");
        }
    }

    mod test_runtime_sessions {
        use super::*;

        fn create_test_session() -> RuntimeSession {
            let source_data = json!({"value": 5});
            let steps = vec![];

            RuntimeSession::new(source_data, steps)
        }

        #[test]
        fn test_session_creation() {
            let _session = create_test_session();
            // We can only test through public APIs
            // In a more complete test we would test processing
        }

        #[test]
        fn test_empty_session_execution() {
            let mut session = create_test_session();

            // Running a session with no steps should succeed
            let result = tokio_test::block_on(session.start());
            assert!(result.is_ok(), "Empty session should execute successfully");

            // The result should match the input since there were no steps
            if let Ok(value) = result {
                assert_eq!(value, json!({"value": 5}));
            }
        }

        #[test]
        fn test_session_with_steps() {
            // Create a test step
            let id_fields = IdFields::new();
            let agent_uuid = Uuid::new_v4();
            let step = Step::new(
                id_fields,
                agent_uuid,
                models::steps::StepType::Python,
                "source['value'] += 10\nresult = source".to_string(),
                "Test Step".to_string(),
                Some("A test step".to_string()),
                0,
                0
            );

            // Create session with our test step
            let source_data = json!({"value": 5});
            let session = RuntimeSession::new(source_data, vec![step]);

            // Since we can't directly access private fields, we'll test
            // the session through its public API in a real test
            // For now this is just a placeholder showing how to create a session with steps
        }
    }

    mod test_id_timestamp_fields {
        use super::*;

        #[test]
        fn test_id_fields_creation() {
            let id = IdFields::new();

            assert_eq!(id.local_id, None);
            assert!(!id.global_uuid.is_empty());
        }

        #[test]
        fn test_id_fields_with_values() {
            let local_id = Some(42u64);
            let global_uuid = "test-uuid".to_string();

            let id = IdFields::with_values(local_id, global_uuid.clone());

            assert_eq!(id.local_id, local_id);
            assert_eq!(id.global_uuid, global_uuid);
        }

        #[test]
        fn test_timestamp_fields_creation() {
            let before = chrono::Local::now().naive_utc();
            std::thread::sleep(Duration::from_millis(5));

            let ts = TimestampFields::new();

            std::thread::sleep(Duration::from_millis(5));
            let after = chrono::Local::now().naive_utc();

            // Timestamps should be between before and after
            assert!(ts.created >= before);
            assert!(ts.created <= after);
            assert!(ts.updated >= before);
            assert!(ts.updated <= after);

            // created and updated should be the same initially
            assert_eq!(ts.created, ts.updated);
        }

        #[test]
        fn test_timestamp_update() {
            let mut ts = TimestampFields::new();
            let created = ts.created;

            // Wait a moment to ensure time difference
            std::thread::sleep(Duration::from_millis(5));

            // Update timestamps
            ts.update();

            // created should not change
            assert_eq!(ts.created, created);

            // updated should be newer
            assert!(ts.updated > ts.created);
        }
    }
}

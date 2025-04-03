#[cfg(test)]
mod tests {
    use crate::{
        exec_python,
        models::{Agent, RuntimeSession, Signal, Step},
        models::steps::StepType,
        IdFields, TimestampFields,
    };
    use serde_json::json;
    use std::time::Duration;
    use uuid::Uuid;
    use std::sync::atomic::Ordering;

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

        fn create_test_agent() -> Agent {
            let id_fields = IdFields::new();
            let timestamps = TimestampFields::new();
            let description = "Test Agent".to_string();
            let accepted_rate = 0.8;

            // Create a simple step that adds 10 to the input value
            let step = Step::new(
                IdFields::new(),
                Uuid::new_v4(),
                StepType::Python,
                "source['value'] += 10\nresult = source".to_string(),
                "Add 10".to_string(),
                Some("Adds 10 to the input value".to_string()),
                0,
                0
            );

            let steps = vec![step];

            Agent::new(
                id_fields,
                timestamps,
                description,
                accepted_rate,
                steps,
                0,
                0
            )
        }

        #[test]
        fn test_new_agent() {
            let mut agent = create_test_agent();

            // Test initial state - agent should start inactive
            let start_result = agent.start();
            if start_result.is_err() {
                println!("Failed to start agent: {:?}", start_result);
            }
            assert!(start_result.is_ok(), "New agent should be able to start");

            // Create a new agent to test the start fails when already started
            let mut another_agent = create_test_agent();

            // Start once
            let first_start = another_agent.start();
            if first_start.is_err() {
                println!("Failed first start: {:?}", first_start);
            }
            assert!(first_start.is_ok());

            // Second start should fail with the expected error
            let second_start = another_agent.start();
            match second_start {
                Ok(_) => panic!("Expected error when starting an already started agent"),
                Err(e) => {
                    println!("Got expected error: {}", e);
                    assert_eq!(e.to_string(), "Can only start from Inactive state");
                }
            }
        }

        #[test]
        fn test_agent_state_transitions() {
            let mut agent = create_test_agent();

            // Test state transitions via available methods

            // Start the agent - should transition to a running state
            let start_result = agent.start();
            if start_result.is_err() {
                println!("Failed to start agent: {:?}", start_result);
            }
            assert!(start_result.is_ok(), "Agent should start successfully");

            // Run the agent with test data
            let source = json!({"value": 5});
            let run_result = tokio_test::block_on(agent.run(source));
            if run_result.is_err() {
                println!("Failed to run agent: {:?}", run_result);
            }
            assert!(run_result.is_ok(), "Agent should run successfully");

            // Verify the step was executed
            if let Ok(session) = run_result {
                println!("Got session result: {:?}", session.last_successful_result);
                assert_eq!(session.last_successful_result.unwrap(), json!({"value": 15}));
            }

            // Stop the agent - should transition back to Inactive
            let stop_result = agent.stop();
            if stop_result.is_err() {
                println!("Failed to stop agent: {:?}", stop_result);
            }
            assert!(stop_result.is_ok(), "Agent should stop successfully");

            // Stopping an inactive agent should fail with the expected error
            let second_stop = agent.stop();
            match second_stop {
                Ok(_) => panic!("Expected error when stopping an inactive agent"),
                Err(e) => {
                    println!("Got expected error: {}", e);
                    assert_eq!(e.to_string(), "Can only stop from a running state");
                }
            }
        }

        #[test]
        fn test_completion_rate() {
            let mut agent = create_test_agent();

            // Create agent with completion history
            agent.completion_count.store(4, Ordering::Relaxed);  // completed
            agent.run_count.store(5, Ordering::Relaxed);         // total runs

            // Start the agent - should transition to a running state
            let start_result = agent.start();
            if start_result.is_err() {
                println!("Failed to start agent: {:?}", start_result);
            }
            assert!(start_result.is_ok(), "Agent should start with good completion rate");

            // Run the agent to verify it works
            let source = json!({"value": 5});
            let run_result = tokio_test::block_on(agent.run(source));
            if run_result.is_err() {
                println!("Failed to run agent: {:?}", run_result);
            }
            assert!(run_result.is_ok(), "Agent should run successfully");

            // Verify the step was executed
            if let Ok(session) = run_result {
                println!("Got session result: {:?}", session.last_successful_result);
                assert_eq!(session.last_successful_result.unwrap(), json!({"value": 15}));
            }
        }
    }

    mod test_steps {
        use super::*;

        fn create_test_step(step_type: StepType) -> Step {
            let id_fields = IdFields::new();
            let agent_uuid = Uuid::new_v4();
            let content = match step_type {
                StepType::Python => "source['value'] += 10\nresult = source".to_string(),
                StepType::Prompt => "Add 10 to the value in the data".to_string()
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
            let step = create_test_step(StepType::Python);

            assert_eq!(step.get_run_count(), 0);
            assert_eq!(step.get_success_count(), 0);
        }

        #[test]
        fn test_step_counters() {
            let step = create_test_step(StepType::Python);

            // Initial counts should be 0
            assert_eq!(step.get_run_count(), 0);
            assert_eq!(step.get_success_count(), 0);

            // We can't directly modify private fields, so we'd need to mock
            // or actually run a step which would increment these counters
            // In real tests you might use test doubles/mocks
        }

        #[test]
        fn test_step_type_behavior() {
            let _python_step = create_test_step(StepType::Python);
            let _prompt_step = create_test_step(StepType::Prompt);

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
                Some(agent),
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
                Some(agent),
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
                StepType::Python,
                "source['value'] += 10\nresult = source".to_string(),
                "Test Step".to_string(),
                Some("A test step".to_string()),
                0,
                0
            );

            // Create session with our test step
            let source_data = json!({"value": 5});
            let _session = RuntimeSession::new(source_data, vec![step]);

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
            let local_id = Some(42i64);
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

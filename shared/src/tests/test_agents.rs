use crate::{
    models::steps::StepType,
    models::{Agent, Step},
    IdFields, TimestampFields,
};
use serde_json::json;

#[test]
fn test_new_agent() {
    let agent = create_test_agent();

    // Test initial state - agent should start inactive
    let start_result = agent.start();
    if start_result.is_err() {
        println!("Failed to start agent: {:?}", start_result);
    }
    assert!(start_result.is_ok(), "New agent should be able to start");

    // Create a new agent to test the start fails when already started
    let another_agent = create_test_agent();

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
    let agent = create_test_agent();

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
        assert_eq!(
            session.last_successful_result.unwrap(),
            json!({"value": 15})
        );
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
    let agent = create_test_agent();

    // Start the agent - should transition to a running state
    let start_result = agent.start();
    if start_result.is_err() {
        println!("Failed to start agent: {:?}", start_result);
    }
    assert!(
        start_result.is_ok(),
        "Agent should start with good completion rate"
    );

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
        assert_eq!(
            session.last_successful_result.unwrap(),
            json!({"value": 15})
        );
    }
}

fn create_test_agent() -> Agent {
    let id_fields = IdFields::new();
    let timestamps = TimestampFields::new();
    let description = "Test Agent".to_string();

    // Create a simple step that adds 10 to the input value
    let step = Step::new(
        IdFields::new(),
        StepType::Python,
        "source['value'] += 10\nresult = source".to_string(),
        Some("Adds 10 to the input value".to_string())
    );

    let steps = vec![step];

    Agent::new(
        id_fields,
        timestamps,
        description,
        steps
    )
}

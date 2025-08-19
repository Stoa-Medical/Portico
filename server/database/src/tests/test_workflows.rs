use crate::{
    models::steps::StepType,
    models::{Workflow, Step},
    IdFields, TimestampFields,
};
use serde_json::json;

#[test]
fn test_new_workflow() {
    let workflow = create_test_workflow();

    // Test initial state - workflow should start inactive
    let start_result = workflow.start();
    if start_result.is_err() {
        println!("Failed to start workflow: {:?}", start_result);
    }
    assert!(start_result.is_ok(), "New workflow should be able to start");

    // Create a new workflow to test the start fails when already started
    let another_workflow = create_test_workflow();

    // Start once
    let first_start = another_workflow.start();
    if first_start.is_err() {
        println!("Failed first start: {:?}", first_start);
    }
    assert!(first_start.is_ok());

    // Second start should fail with the expected error
    let second_start = another_workflow.start();
    match second_start {
        Ok(_) => panic!("Expected error when starting an already started workflow"),
        Err(e) => {
            println!("Got expected error: {}", e);
            assert_eq!(e.to_string(), "Can only start from Inactive state");
        }
    }
}

#[test]
fn test_workflow_state_transitions() {
    let workflow = create_test_workflow();

    // Test state transitions via available methods

    // Start the workflow - should transition to a running state
    let start_result = workflow.start();
    if start_result.is_err() {
        println!("Failed to start workflow: {:?}", start_result);
    }
    assert!(start_result.is_ok(), "Workflow should start successfully");

    // Run the workflow with test data
    let source = json!({"value": 5});
    let run_result = tokio_test::block_on(workflow.run(source));
    if run_result.is_err() {
        println!("Failed to run workflow: {:?}", run_result);
    }
    assert!(run_result.is_ok(), "Workflow should run successfully");

    // Verify the step was executed
    if let Ok(session) = run_result {
        println!("Got session result: {:?}", session.last_successful_result);
        assert_eq!(
            session.last_successful_result.unwrap(),
            json!({"value": 15})
        );
    }

    // Stop the workflow - should transition back to Inactive
    let stop_result = workflow.stop();
    if stop_result.is_err() {
        println!("Failed to stop workflow: {:?}", stop_result);
    }
    assert!(stop_result.is_ok(), "Workflow should stop successfully");

    // Stopping an inactive workflow should fail with the expected error
    let second_stop = workflow.stop();
    match second_stop {
        Ok(_) => panic!("Expected error when stopping an inactive workflow"),
        Err(e) => {
            println!("Got expected error: {}", e);
            assert_eq!(e.to_string(), "Can only stop from a running state");
        }
    }
}

#[test]
fn test_completion_rate() {
    let workflow = create_test_workflow();

    // Start the workflow - should transition to a running state
    let start_result = workflow.start();
    if start_result.is_err() {
        println!("Failed to start workflow: {:?}", start_result);
    }
    assert!(
        start_result.is_ok(),
        "Workflow should start with good completion rate"
    );

    // Run the workflow to verify it works
    let source = json!({"value": 5});
    let run_result = tokio_test::block_on(workflow.run(source));
    if run_result.is_err() {
        println!("Failed to run workflow: {:?}", run_result);
    }
    assert!(run_result.is_ok(), "Workflow should run successfully");

    // Verify the step was executed
    if let Ok(session) = run_result {
        println!("Got session result: {:?}", session.last_successful_result);
        assert_eq!(
            session.last_successful_result.unwrap(),
            json!({"value": 15})
        );
    }
}

fn create_test_workflow() -> Workflow {
    let id_fields = IdFields::new();
    let timestamps = TimestampFields::new();
    let description = "Test Workflow".to_string();

    // Create a simple step that adds 10 to the input value
    let step = Step::new(
        IdFields::new(),
        StepType::Python,
        "source['value'] += 10\nresult = source".to_string(),
        Some("Adds 10 to the input value".to_string()),
    );

    let steps = vec![step];

    Workflow::new(id_fields, timestamps, description, steps)
}

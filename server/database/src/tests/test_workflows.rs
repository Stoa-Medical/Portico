use crate::{
    models::steps::StepType,
    models::{Workflow, Step},
    IdFields, TimestampFields,
};
use serde_json::json;

#[test]
fn test_new_workflow() {
    let mut workflow = create_test_workflow();

    // Test initial state - workflow should start inactive
    let start_result = workflow.start();
    assert!(start_result.is_ok(), "New workflow should be able to start");

    // Second start should fail with the expected error
    let second_start = workflow.start();
    match second_start {
        Ok(_) => panic!("Expected error when starting an already started workflow"),
        Err(e) => {
            assert_eq!(e.to_string(), "Can only start from Inactive state");
        }
    }
}

#[test]
fn test_workflow_state_transitions() {
    let mut workflow = create_test_workflow();

    // Start the workflow
    let start_result = workflow.start();
    assert!(start_result.is_ok(), "Workflow should start successfully");

    // Stop the workflow
    let stop_result = workflow.stop();
    assert!(stop_result.is_ok(), "Workflow should stop successfully");

    // Stopping an inactive workflow should fail
    let second_stop = workflow.stop();
    match second_stop {
        Ok(_) => panic!("Expected error when stopping an inactive workflow"),
        Err(e) => {
            assert_eq!(e.to_string(), "Can only stop from a running state");
        }
    }
}

fn create_test_workflow() -> Workflow {
    let id_fields = IdFields::new();
    let timestamps = TimestampFields::new();

    Workflow::new(
        id_fields,
        timestamps,
        Some("Test Workflow".to_string()),
        Some("A test workflow".to_string()),
    )
}

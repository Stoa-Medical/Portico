use crate::{
    models::{Agent, Signal, SignalType},
    IdFields, TimestampFields,
};
use serde_json::json;
use uuid::Uuid;

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
        vec![]
    );

    let initial_data = Some(json!({"value": 5}));

    Signal::new(
        id_fields,
        user_uuid,
        Some(agent),
        SignalType::Fyi,
        initial_data,
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
        vec![]
    );

    // Create signal with no initial data
    let mut signal = Signal::new(
        id_fields,
        user_uuid,
        Some(agent),
        SignalType::Fyi,
        None, // No data
    );

    // Processing should fail without data
    let process_result = tokio_test::block_on(signal.process());
    assert!(process_result.is_err(), "Process should fail without data");
}

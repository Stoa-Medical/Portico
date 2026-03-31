use crate::{
    models::{Signal, SignalType},
    IdFields,
};
use serde_json::json;
use uuid::Uuid;

fn create_test_signal() -> Signal {
    let id_fields = IdFields::new();
    let user_uuid = Uuid::new_v4().to_string();
    let initial_data = Some(json!({"value": 5}));

    Signal::new(
        id_fields,
        user_uuid,
        None,  // workflow_id
        None,  // initiator_agent_id
        SignalType::Fyi,
        initial_data,
    )
}

#[test]
fn test_signal_creation() {
    let signal = create_test_signal();
    assert_eq!(signal.signal_type, SignalType::Fyi);
    assert!(signal.initial_data.is_some());
    assert!(signal.source.is_none());
    assert!(signal.idempotency_key.is_none());
}

#[test]
fn test_signal_without_data() {
    let id_fields = IdFields::new();
    let user_uuid = Uuid::new_v4().to_string();

    let signal = Signal::new(
        id_fields,
        user_uuid,
        None,
        None,
        SignalType::Fyi,
        None,
    );

    assert!(signal.initial_data.is_none());
    assert!(!signal.is_processed());
}

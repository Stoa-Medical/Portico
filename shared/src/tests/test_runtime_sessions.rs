use crate::{
    models::{RuntimeSession, Step},
    models::steps::StepType,
    IdFields,
};
use serde_json::json;

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
    let step = Step::new(
        id_fields,
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

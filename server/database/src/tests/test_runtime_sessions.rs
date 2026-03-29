use crate::{
    models::steps::StepType,
    models::{RuntimeSession, Step},
    IdFields,
};
use serde_json::json;

fn create_test_session() -> RuntimeSession {
    let source_data = json!({"value": 5});
    let steps = vec![];

    RuntimeSession::new(source_data, steps, None)
}

#[test]
fn test_session_creation() {
    let session = create_test_session();
    assert!(session.data_quality_score.is_none());
    assert!(session.signal_id.is_none());
}

#[test]
fn test_empty_session_execution() {
    let mut session = create_test_session();

    // Running a session now returns an error directing to engine dispatch
    let result = tokio_test::block_on(session.start());
    assert!(result.is_err(), "Session execution should direct to engine dispatch");
}

#[test]
fn test_session_with_steps() {
    // Create a test step with the new config-based API
    let id_fields = IdFields::new();
    let step = Step::new(
        id_fields,
        StepType::Python,
        Some(json!({"script": "source['value'] += 10\nresult = source"})),
        Some("A test step".to_string()),
    );

    // Create session with our test step
    let source_data = json!({"value": 5});
    let session = RuntimeSession::new(source_data, vec![step], None);

    assert_eq!(session.steps.len(), 1);
    assert!(session.data_quality_score.is_none());
    assert!(session.signal_id.is_none());
}

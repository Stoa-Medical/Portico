use crate::{exec_python, IdFields, TimestampFields};
use serde_json::json;
use std::time::Duration;

#[test]
fn test_exec_python() {
    // Test a simple Python function that adds numbers
    let source = json!({"a": 5, "b": 7});
    let python_code = r#"
result = {"sum": source["a"] + source["b"]}
"#;

    let result = exec_python(source, python_code);
    assert!(
        result.is_ok(),
        "{}",
        format!("Python execution should succeed, got: {:?}", result)
    );

    if let Ok(value) = result {
        assert_eq!(value, json!({"sum": 12}));
    }
}

#[test]
fn test_id_fields_creation() {
    let id = IdFields::new();

    assert_eq!(id.local_id, None);
    assert!(!id.global_uuid.is_empty());
}

#[test]
fn test_id_fields_with_values() {
    let local_id = Some(42i32);
    let global_uuid = "test-uuid".to_string();

    let id = IdFields::with_values(local_id, global_uuid.clone());

    assert_eq!(id.local_id, local_id);
    assert_eq!(id.global_uuid, global_uuid);
}

#[test]
fn test_timestamp_fields_creation() {
    let before = chrono::Utc::now();
    std::thread::sleep(Duration::from_millis(5));

    let ts = TimestampFields::new();

    std::thread::sleep(Duration::from_millis(5));
    let after = chrono::Utc::now();

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

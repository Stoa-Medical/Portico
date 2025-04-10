use crate::{models::steps::StepType, models::Step, IdFields};

fn create_test_step(step_type: StepType) -> Step {
    let id_fields = IdFields::new();
    let content = match step_type {
        StepType::Python => "source['value'] += 10\nresult = source".to_string(),
        StepType::Prompt => "Add 10 to the value in the data".to_string(),
    };

    Step::new(
        id_fields,
        step_type,
        content,
        "Test Step".to_string(),
        Some("A test step that adds 10 to the input value".to_string()),
        0,
        0,
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

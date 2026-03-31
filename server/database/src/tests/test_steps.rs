use crate::{models::steps::StepType, models::Step, IdFields};
use serde_json::json;

fn create_test_step(step_type: StepType) -> Step {
    let id_fields = IdFields::new();
    let config = match &step_type {
        StepType::Python => Some(json!({"script": "source['value'] += 10\nresult = source"})),
        StepType::LLM => Some(json!({"prompt": "Add 10 to the value in the data"})),
        StepType::Transform => Some(json!({"mapping": "identity"})),
        StepType::Validate => Some(json!({"schema": "default"})),
        StepType::FHIR => Some(json!({"resource_type": "Patient"})),
    };

    Step::new(
        id_fields,
        step_type,
        config,
        Some("A test step".to_string()),
    )
}

#[test]
fn test_create_step() {
    let _python_step = create_test_step(StepType::Python);
    let _llm_step = create_test_step(StepType::LLM);
    let _transform_step = create_test_step(StepType::Transform);
    let _validate_step = create_test_step(StepType::Validate);
    let _fhir_step = create_test_step(StepType::FHIR);
}

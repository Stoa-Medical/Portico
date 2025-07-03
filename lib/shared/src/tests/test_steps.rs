use crate::{models::steps::StepType, models::Step, IdFields};

fn create_test_step(step_type: StepType) -> Step {
    let id_fields = IdFields::new();
    let content = match &step_type {
        StepType::Python => "source['value'] += 10\nresult = source".to_string(),
        StepType::Prompt(_) => "Add 10 to the value in the data".to_string(),
        StepType::WebScrape => "https://example.com".to_string(),
    };

    Step::new(
        id_fields,
        step_type,
        content,
        Some("A test step that adds 10 to the input value".to_string()),
    )
}

#[test]
fn test_create_step() {
    let _python_step = create_test_step(StepType::Python);
    let _prompt_step = create_test_step(StepType::Prompt(
        "meta-llama/Llama-3.3-70B-Instruct-Turbo".to_string(),
    ));
    let _webscrape_step = create_test_step(StepType::WebScrape);

    // In real tests, we'd test different behavior based on step type
    // For example, the Python step should execute Python code
    // and the Prompt step should call the LLM
    // Here we'd mock those dependencies
}

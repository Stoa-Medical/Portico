use super::types::{Step, StepType};
use crate::PythonRuntime;
use anyhow::{anyhow, Result};
use serde_json::{Value, Map};

// Define standard output keys for all step types
pub const STEP_OUTPUT_RESPONSE_KEY: &str = "response";
pub const STEP_OUTPUT_DATA_KEY: &str = "data";
pub const STEP_OUTPUT_STATUS_KEY: &str = "status";
pub const STEP_OUTPUT_ERROR_KEY: &str = "error";
pub const STEP_OUTPUT_TYPE_KEY: &str = "output_type";
pub const STEP_OUTPUT_SOURCE_KEY: &str = "source_step";

impl Step {
    /// Generates a Python function with the standardized signature for execution in a PythonRuntime
    pub fn to_python_function(&self) -> String {
        let func_name = self.python_function_name();
        let docstring = format!(
            "\"\"\"\n    {}\n    \n    Args:\n        source: Input data dictionary from previous step\n        \n    Returns:\n        dict: Output data to pass to next step\n    \"\"\"",
            self.description.as_deref().unwrap_or("No description provided")
        );

        format!(
            r#"def {}(source):
    {}
    # Step implementation
    result = source  # Default pass-through

{}

    return result"#,
            func_name,
            docstring,
            // Indent all lines with 4 spaces for proper Python indentation
            self.step_content
                .lines()
                .map(|line| format!("    {}", line))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Returns the standard Python function name for this step
    pub fn python_function_name(&self) -> String {
        format!("step_{}", self.identifiers.global_uuid.replace("-", "_"))
    }

    /// Ensures the output is in standardized dictionary format
    fn standardize_output(&self, output: Value) -> Value {
        match output {
            // Already a dictionary/object, add standard fields if missing
            Value::Object(mut map) => {
                // Add source step info if not present
                if !map.contains_key(STEP_OUTPUT_SOURCE_KEY) {
                    map.insert(
                        STEP_OUTPUT_SOURCE_KEY.to_string(),
                        Value::String(self.identifiers.global_uuid.clone()),
                    );
                }

                // Add step type info if not present
                if !map.contains_key(STEP_OUTPUT_TYPE_KEY) {
                    map.insert(
                        STEP_OUTPUT_TYPE_KEY.to_string(),
                        Value::String(self.step_type.as_str().to_string()),
                    );
                }

                // Add status if not present
                if !map.contains_key(STEP_OUTPUT_STATUS_KEY) {
                    map.insert(
                        STEP_OUTPUT_STATUS_KEY.to_string(),
                        Value::String("success".to_string()),
                    );
                }

                Value::Object(map)
            }
            // String output (typically from prompt steps)
            Value::String(text) => {
                let mut map = Map::new();
                map.insert(
                    STEP_OUTPUT_RESPONSE_KEY.to_string(),
                    Value::String(text),
                );
                map.insert(
                    STEP_OUTPUT_SOURCE_KEY.to_string(),
                    Value::String(self.identifiers.global_uuid.clone()),
                );
                map.insert(
                    STEP_OUTPUT_TYPE_KEY.to_string(),
                    Value::String(self.step_type.as_str().to_string()),
                );
                map.insert(
                    STEP_OUTPUT_STATUS_KEY.to_string(),
                    Value::String("success".to_string()),
                );
                Value::Object(map)
            }
            // Arrays or other non-dictionary values
            other => {
                let mut map = Map::new();
                map.insert(STEP_OUTPUT_DATA_KEY.to_string(), other);
                map.insert(
                    STEP_OUTPUT_SOURCE_KEY.to_string(),
                    Value::String(self.identifiers.global_uuid.clone()),
                );
                map.insert(
                    STEP_OUTPUT_TYPE_KEY.to_string(),
                    Value::String(self.step_type.as_str().to_string()),
                );
                map.insert(
                    STEP_OUTPUT_STATUS_KEY.to_string(),
                    Value::String("success".to_string()),
                );
                Value::Object(map)
            }
        }
    }

    /// Runs the step with fresh context
    pub async fn run(
        &self,
        source_data: Value,
        step_idx: usize,
        runtime: Option<&PythonRuntime>,
    ) -> Result<Value> {
        let raw_result = match &self.step_type {
            StepType::Prompt(llm_model) => {
                match crate::call_llm(&self.step_content, source_data.clone(), Some(llm_model.clone()))
                    .await
                {
                    Ok(res_str) => Ok(Value::String(res_str)),
                    Err(err) => Err(anyhow!("Step {} failed: {}", step_idx, err))
                }
            }
            StepType::Python => {
                // For Python steps, require a runtime
                if let Some(rt) = runtime {
                    rt.execute_step(&self.identifiers.global_uuid, source_data.clone())
                } else {
                    Err(anyhow!(
                        "Python step {} requires a runtime to execute",
                        step_idx
                    ))
                }
            }
            StepType::WebScrape => {
                // For WebScrape steps, the step_content should contain the URL to scrape
                let url = self.step_content.trim();
                if url.is_empty() {
                    return Err(anyhow!("WebScrape step {} (UUID: {}) has empty URL",
                                       step_idx,
                                       self.identifiers.global_uuid));
                }

                // Call the web scraping function
                match crate::scrape_webpage(url).await {
                    Ok(result) => Ok(result),
                    Err(err) => Err(anyhow!("WebScrape step {} (UUID: {}) failed: {}",
                                            step_idx,
                                            self.identifiers.global_uuid,
                                            err.to_string()))
                }
            }
        };

        // Return raw output
        match raw_result {
            Ok(output) => Ok(output),
            Err(err) => {
                // Create standardized error output
                let mut error_map = Map::new();
                error_map.insert(
                    STEP_OUTPUT_ERROR_KEY.to_string(),
                    Value::String(err.to_string())
                );
                error_map.insert(
                    STEP_OUTPUT_STATUS_KEY.to_string(),
                    Value::String("error".to_string()),
                );
                error_map.insert(
                    STEP_OUTPUT_SOURCE_KEY.to_string(),
                    Value::String(self.identifiers.global_uuid.clone()),
                );
                error_map.insert(
                    STEP_OUTPUT_TYPE_KEY.to_string(),
                    Value::String(self.step_type.as_str().to_string()),
                );

                // Still propagate the original error
                Err(err)
            }
        }
    }
}

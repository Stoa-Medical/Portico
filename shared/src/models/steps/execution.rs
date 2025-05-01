use super::types::{Step, StepType};
use crate::PythonRuntime;
use anyhow::{anyhow, Result};
use serde_json::Value;

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

    /// Runs the step with fresh context
    pub async fn run(
        &self,
        source_data: Value,
        step_idx: usize,
        runtime: Option<&PythonRuntime>,
    ) -> Result<Value> {
        match &self.step_type {
            StepType::Prompt(llm_model) => {
                match crate::call_llm(&self.step_content, source_data, Some(llm_model.clone()))
                    .await
                {
                    Ok(res_str) => Ok(Value::String(res_str)),
                    Err(err) => Err(anyhow!("Step {} failed: {}", step_idx, err)),
                }
            }
            StepType::Python => {
                // For Python steps, require a runtime
                if let Some(rt) = runtime {
                    rt.execute_step(&self.identifiers.global_uuid, source_data)
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
                    return Err(anyhow!("WebScrape step {} has empty URL", step_idx));
                }

                // Call the web scraping function
                match crate::scrape_webpage(url).await {
                    Ok(result) => Ok(result),
                    Err(err) => Err(anyhow!("WebScrape step {} failed: {}", step_idx, err)),
                }
            }
        }
    }
}

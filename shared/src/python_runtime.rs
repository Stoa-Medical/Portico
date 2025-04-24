use crate::models::Step;
use anyhow::{anyhow, Result};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::CString;

/// Manages a Python execution environment for Agents
pub struct PythonRuntime {
    /// Python module containing all step functions for this agent
    module: Py<PyModule>,
    /// Maps step UUIDs to their Python function names
    step_functions: HashMap<String, String>,
}

impl PythonRuntime {
    /// Create a new Python runtime with a unique module name
    pub fn new(name: &str) -> Result<Self> {
        Python::with_gil(|py| {
            let module_name = format!("agent_{}", name.replace("-", "_"));
            let module = PyModule::new(py, &module_name)?;

            // Import common modules - just make it available in Python context
            let _ = py.import("json")?;

            Ok(Self {
                module: module.into(),
                step_functions: HashMap::new(),
            })
        })
    }

    /// Add a step to the runtime
    pub fn add_step(&mut self, step: &Step) -> Result<()> {
        if !step.is_python_step() {
            return Ok(()); // Skip non-Python steps
        }

        Python::with_gil(|py| {
            // Get the Python function code for this step
            let func_code = step.to_python_function();
            let func_name = step.python_function_name();

            // Get a reference to the module
            let module_ref = &self.module.bind(py);

            // Add the function to the module
            let locals = module_ref.dict();

            // Convert to CString for py.run
            let code_cstring = CString::new(func_code.as_bytes())?;
            py.run(code_cstring.as_c_str(), None, Some(&locals))?;

            // Store the function name mapped to the step UUID
            self.step_functions
                .insert(step.identifiers.global_uuid.clone(), func_name);

            Ok(())
        })
    }

    /// Execute a step with the given input data
    pub fn execute_step(&self, step_uuid: &str, input: Value) -> Result<Value> {
        let func_name = self
            .step_functions
            .get(step_uuid)
            .ok_or_else(|| anyhow!("Step function not found: {}", step_uuid))?;

        Python::with_gil(|py| {
            // Get a reference to the module
            let module_ref = &self.module.bind(py);

            // Get the json module
            let py_json = py.import("json")?;

            // Convert input to Python object
            let json_str = serde_json::to_string(&input)?;
            let py_input = py_json.getattr("loads")?.call1((json_str,))?;

            // Call the function
            if let Ok(func) = module_ref.getattr(func_name) {
                let result = func.call1((py_input,))?;

                // Convert the result back to Rust
                let py_json_str = py_json.getattr("dumps")?.call1((result,))?;
                let rust_json_str: String = py_json_str.extract()?;
                let rust_value: Value = serde_json::from_str(&rust_json_str)?;

                Ok(rust_value)
            } else {
                Err(anyhow!("Function not found in module: {}", func_name))
            }
        })
    }
}

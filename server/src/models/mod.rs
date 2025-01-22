/// Module for defining and managing Agents
pub mod agents;
/// Module for running Steps + data
pub mod session;
/// Module for defining and running Steps
pub mod steps;
// /// Module for interfacing with the database
// pub mod db;


/// Tests for all the modules in this subdirectory
#[cfg(test)]
mod tests {
    use serde_json::Value;

    /// Tests for the `Step` library
    mod test_steps {
        use super::super::steps::{Step, StepAction};
        use serde_json::json; 

        #[test]
        fn test_step_new() {
            let step = Step::new(
                "test_step".to_string(),
                StepAction::Python("print('hello')".to_string())
            );
            
            assert_eq!(step.name, "test_step");
            assert_eq!(step.get_run_count(), 0);  // Use getter instead of direct comparison
            match step.instruction {
                StepAction::Python(code) => assert_eq!(code, "print('hello')"),
                _ => panic!("Wrong instruction type"),
            }
        }
    
        #[tokio::test]
        async fn test_python_step_execution() {
            let step = Step::new(  // No need for mut
                "python_test".to_string(),
                StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data['value'] * 2
"#.to_string())
            );
    
            let input = json!({"value": 21});
            let result = step.run(input, 0).await.unwrap().unwrap();
            
            assert_eq!(result, json!(42));
            assert_eq!(step.get_run_count(), 1);
        }
    
        #[tokio::test]
        async fn test_python_step_no_result() {
            let step = Step::new(  // No need for mut
                "no_result_test".to_string(),
                StepAction::Python("print('hello')".to_string())
            );
    
            let input = json!({"value": 21});
            let result = step.run(input, 0).await.unwrap();
            
            assert!(result.is_none());
            assert_eq!(step.get_run_count(), 1);
        }
    
        #[tokio::test]
        async fn test_python_step_error() {
            let step = Step::new(  // No need for mut
                "error_test".to_string(),
                StepAction::Python("invalid python code".to_string())
            );
    
            let input = json!({"value": 21});
            let result = step.run(input, 0).await;
            
            assert!(result.is_err());
            assert_eq!(step.get_run_count(), 1);
        }
    
        #[tokio::test]
        async fn test_step_multiple_runs() {
            let step = Step::new(
                "multiple_runs".to_string(),
                StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data['value'] * 2
"#.to_string())
            );
    
            let input = json!({"value": 21});
            
            // Run multiple times
            step.run(input.clone(), 0).await.unwrap();
            step.run(input.clone(), 1).await.unwrap();
            step.run(input.clone(), 2).await.unwrap();
            
            assert_eq!(step.get_run_count(), 3);
        }
    
        #[tokio::test]
        async fn test_step_count_on_error() {
            let step = Step::new(
                "error_count".to_string(),
                StepAction::Python("invalid python code".to_string())
            );
    
            let input = json!({"value": 21});
            
            // Even failed runs should increment the counter
            let _ = step.run(input.clone(), 0).await;
            let _ = step.run(input.clone(), 1).await;
            
            assert_eq!(step.get_run_count(), 2);
        }
    
        // // Note: This test will need a mock for call_llm to work properly
        // #[tokio::test]
        // #[ignore] // Ignore by default since it needs LLM configuration
        // async fn test_prompt_step() {
        //     let mut step = Step::new(
        //         "prompt_test".to_string(),
        //         StepAction::Prompt("Test prompt".to_string())
        //     );
    
        //     let input = json!({"value": "test"});
        //     let result = step.run(input).await.unwrap().unwrap();
            
        //     assert!(matches!(result, Value::String(_)));
        //     assert_eq!(step.run_count, 1);
        // }
    }

    mod test_session {
        use crate::models::session::Session;
        use crate::models::steps::{Step, StepAction};
        use serde_json::json;
    
        #[tokio::test]
        async fn test_session_basic() {
            let steps = vec![
                Step::new(
                    "step1".to_string(),
                    StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data['value'] * 2
"#.to_string())
            ),
            Step::new(
                "step2".to_string(),
                StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data + 1
"#.to_string())
                ),
            ];
    
            let input = json!({"value": 21});
            let mut session = Session::new(&steps, input);
            
            let result = session.run_all(true).await.unwrap();
            assert!(result.is_some());
            assert!(session.completed);
            assert_eq!(steps[0].get_run_count(), 1);
            assert_eq!(steps[1].get_run_count(), 1);
        }
    
        #[tokio::test]
        async fn test_session_step_by_step() {
            let steps = vec![
                Step::new(
                    "step1".to_string(),
                    StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data['value'] * 2
"#.to_string())
            ),
            Step::new(
                "step2".to_string(),
                StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data + 1
"#.to_string())
                ),
            ];
    
            let input = json!({"value": 21});
            let mut session = Session::new(&steps, input);
            
            // Run first step
            let result1 = session.run_steps(1, true).await.unwrap();
            assert!(result1.is_some());
            assert!(!session.completed);
            assert_eq!(steps[0].get_run_count(), 1);
            assert_eq!(steps[1].get_run_count(), 0);
    
            // Run second step
            let result2 = session.run_steps(1, true).await.unwrap();
            assert!(result2.is_some());
            assert!(session.completed);
            assert_eq!(steps[0].get_run_count(), 1);
            assert_eq!(steps[1].get_run_count(), 1);
        }
    
        #[tokio::test]
        async fn test_session_retry_on_error() {
            let steps = vec![
                Step::new(
                    "step1".to_string(),
                StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data['value'] * 2
"#.to_string())
                ),
                Step::new(
                    "step2".to_string(),
                    StepAction::Python("invalid python code".to_string())
                ),
            ];
    
            let input = json!({"value": 21});
            let mut session = Session::new(&steps, input);
            
            // First attempt fails at step2
            let result1 = session.run_all(true).await;
            println!("After first run: {:?}", result1);
            println!("Current idx: {}", session.curr_idx);
            assert!(result1.is_err());
            
            // Retry the failed step
            let result2 = session.run_all(true).await;
            println!("After second run: {:?}", result2);
            println!("Current idx: {}", session.curr_idx);
            assert!(result2.is_err());
            
            // Check run counts
            assert_eq!(steps[0].get_run_count(), 1); // Expect this to run once
            assert_eq!(steps[1].get_run_count(), 2); // Second step fails twice
        }
    }
}
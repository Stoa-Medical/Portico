/// Module for defining and managing Agents
pub mod agents;
pub use agents::Agent;
/// Module for running Steps + data (in a RuntimeSession, abbrv. rts)
pub mod runtime;
pub use runtime::RuntimeSession;
/// Module for defining and running Steps
pub mod steps;
pub use steps::Step;


/// Tests for all the modules in this subdirectory
#[cfg(test)]
mod tests {

    mod test_steps {
        use crate::models::steps::{Step, StepAction};
        use serde_json::json;
        use crate::DataSource;

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
    
            let input = DataSource::Json(json!({"value": 21}));
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
    
            let input =  DataSource::Json(json!({"value": 21}));
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
    
            let input =  DataSource::Json(json!({"value": 21}));
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
    
            let input =  DataSource::Json(json!({"value": 21}));
            
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
    
            let input =  DataSource::Json(json!({"value": 21}));
            
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
        use crate::models::runtime::RuntimeSession;
        use crate::models::steps::{Step, StepAction};
        use crate::DataSource;
        use serde_json::json;
    
        #[tokio::test]
        async fn test_session_basic() {
            let mut steps = vec![
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
    
            let input = DataSource::Json(json!({"value": 21}));
            
            {
                let mut rts = RuntimeSession::new(&mut steps, input);
                let result = rts.run_all(true).await.unwrap();
                assert!(result.is_some());
                assert!(rts.is_completed());
            }
    
            // Verify final state
            assert_eq!(steps[0].get_run_count(), 1);
            assert_eq!(steps[1].get_run_count(), 1);
        }
        
        #[tokio::test]
        async fn test_session_checkpoints() {
                let mut steps = vec![
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

                let input = DataSource::Json(json!({"value": 21}));
                let mut checkpoint = None;
                
                // First RuntimeSession - run until interruption
                {
                    let mut rts = RuntimeSession::new(&mut steps, input.clone());
                    let result = rts.run_n_steps(1, true).await.unwrap();
                    assert!(result.is_some());
                    checkpoint = rts.save_checkpoint();
                }

                // Verify intermediate state
                assert_eq!(steps[0].get_run_count(), 1);
                assert_eq!(steps[0].get_success_count(), 1);
                assert_eq!(steps[1].get_run_count(), 0);
                
                // Resume RuntimeSession from checkpoint
                {
                    let mut rts = RuntimeSession::resume_from_checkpoint(&mut steps, input, checkpoint.unwrap());
                    let result = rts.run_all(true).await.unwrap();
                    assert!(result.is_some());
                    assert!(rts.is_completed());
                }

                // Verify final state
                assert_eq!(steps[0].get_run_count(), 1); // First step shouldn't run again
                assert_eq!(steps[0].get_success_count(), 1);
                assert_eq!(steps[1].get_run_count(), 1); // Only second step runs
                assert_eq!(steps[1].get_success_count(), 1);
        }
    }
    mod test_agents {
        use crate::models::agents::{Agent, AgentType};
        use crate::models::steps::{Step, StepAction};
        use crate::DataSource;
        use serde_json::json;
    
        fn create_test_agent(err_rate: f32, agent_type: AgentType) -> Agent {
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
    
            Agent::new(
                "Test Agent".to_string(),
                err_rate,
                steps,
                agent_type
            )
        }
    
        #[test]
        fn test_agent_state_transitions() {
            let mut agent = create_test_agent(0.1, AgentType::Reactor);
            
            // Test initial state
            assert!(agent.start().is_ok());
            
            // Test invalid transitions
            assert!(agent.start().is_err()); // Can't start twice
            
            // Test check operation
            assert!(agent.check().is_ok()); // Should be stable initially
            
            // Test stop operation
            assert!(agent.stop().is_ok());
            assert!(agent.check().is_err()); // Can't check when inactive
        }
    
        #[tokio::test]
        async fn test_agent_reactor_behavior() {
            let mut agent = create_test_agent(0.1, AgentType::Reactor);
            agent.start().unwrap();
            
            let input = DataSource::Json(json!({"value": 21}));
            let result = agent.run(input).await;
            
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), json!(43)); // (21 * 2) + 1
        }
    
        #[tokio::test]
        async fn test_agent_actor_behavior() {
            let mut agent = create_test_agent(
                0.1, 
                AgentType::Actor("0 0 * * * *".to_string())
            );
            agent.start().unwrap();
            
            let input = DataSource::Json(json!({"value": 21}));
            let result = agent.run(input).await;
            
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), json!(43)); // (21 * 2) + 1
        }
    
        #[tokio::test]
        async fn test_agent_error_handling() {
            let steps = vec![
                Step::new(
                    "step1".to_string(),
                    StepAction::Python("invalid python code".to_string())
                ),
            ];
    
            let mut agent = Agent::new(
                "Error Test Agent".to_string(),
                0.1,
                steps,
                AgentType::Reactor
            );
    
            agent.start().unwrap();
            
            let input = DataSource::Json(json!({"value": 21}));
            let result = agent.run(input).await;
            
            assert!(result.is_err());
            assert!(agent.check().is_err()); // Should be unstable
        }
    
        #[test]
        fn test_error_rate_calculation() {
            let steps = vec![
                Step::new(
                    "step1".to_string(),
                    StepAction::Python("print('test')".to_string())
                ),
            ];
    
            let agent = Agent::new(
                "Error Rate Test".to_string(),
                0.1,
                steps,
                AgentType::Reactor
            );
    
            // Initially should have 0 error rate
            assert_eq!(agent.get_err_rate(), 0.0);
        }
    
        #[tokio::test]
        async fn test_agent_stability_threshold() {
            let mut agent = create_test_agent(0.5, AgentType::Reactor); // 50% error threshold
            agent.start().unwrap();
            
            let input = DataSource::Json(json!({"value": 21}));
            let result = agent.run(input).await;

            assert!(result.is_ok());

            let check_result = agent.check();
            assert!(check_result.is_ok()); // Should be stable
    
            // Now create an agent with very low error threshold
            let mut strict_agent = create_test_agent(0.01, AgentType::Reactor); // 1% error threshold
            strict_agent.start().unwrap();
            
            // Run with invalid input to cause errors
            let invalid_input = DataSource::Json(json!({"wrong_key": 21}));
            let result = strict_agent.run(invalid_input).await;
            
            assert!(result.is_err());
            assert!(strict_agent.check().is_err()); // Should be unstable
        }
    }
}
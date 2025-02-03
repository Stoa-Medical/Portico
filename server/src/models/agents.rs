/// Class for defining Agents
///   An Agent is a listener that can receive + react to data
///   An Agent can take Steps to achieve its goal (based on heuristics)
///     Steps are python code, and the scaffolding is defined in `steps.rs`

use super::steps::Step;
use crate::{CanAct, CanReact, DataSource};
use super::jobs::Job;
use super::runtime::RuntimeSession;

use std::collections::HashMap;
use serde_json::Value;
use anyhow::Result;
use uuid::Uuid;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Agents are units of action
#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    /// The UUID identifier
    id: String,
    /// What does this agent do (as described by a human)?
    description: String,
    /// The state of the agent
    agent_state: AgentState,
    /// Whether the agent acts (on its own on a schedule) or reacts (waits and responds to event)
    agent_type: AgentType,
    /// The steps the agent should take
    steps: Vec<Step>,
    /// The allowed error rate before flagging as `AgentState::Unstable`
    accepted_err_rate: f32,
}

/// Different states for actor to be in. State diagram:
/// ```plain
///                    ┌────────────────┐
///                    │                │
///          (start)   v      (run)     |
///  Inactive ──────► Waiting ──────► Running ──┐
///      ▲             │   ▲            ▲       |
///      │             │   │ (check)    | (run) |
///      │             │   └───────► Unstable ◄─┘
///      |             v                |
///      └───────── Stopping ◄──────────┘
/// ```
#[derive(Debug, PartialEq,  Clone, Serialize, Deserialize)]
pub enum AgentState {
    Inactive,
    Waiting,
    Running,
    Unstable,
    Stopping
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    /// Performs a specific action based on a CRON schedule
    Actor(String),
    Reactor
}

impl Agent {
    pub fn new(description: String, accepted_err_rate: f32, steps: Vec<Step>, agent_type: AgentType) -> Self {
        // Start all agents in a waiting state
        Self {
            id: Uuid::new_v4().to_string(),
            description,
            agent_state: AgentState::Inactive,
            steps,
            agent_type,
            accepted_err_rate
        }
    }
    
    pub fn start(&mut self) -> Result<()> {
        match self.agent_state {
            AgentState::Inactive => {
                self.agent_state = AgentState::Waiting;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Can only start from Inactive state"))
        }
    }

    /// Returns Ok and sets to `Waiting` if stable, else Err and sets to `Unstable`
    pub fn check(&mut self) -> Result<()> {
        match self.agent_state {
            AgentState::Waiting | AgentState::Unstable => {
                let curr_err_rate = self.get_err_rate();
                let is_stable = curr_err_rate <= self.accepted_err_rate;
                
                // Update state based on stability
                self.agent_state = if is_stable {
                    AgentState::Waiting
                } else {
                    AgentState::Unstable
                };
    
                // Return result with appropriate message
                if is_stable {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "Expected error rate <= {:.6}, got {:.6}", 
                        self.accepted_err_rate, 
                        curr_err_rate
                    ))
                }
            },
            _ => Err(anyhow::anyhow!("Can only check from Waiting/Unstable state"))
        }
    }

    pub async fn run(&mut self, source: DataSource, job: Option<&mut Job>) -> Result<Value> {
        match self.agent_state {
            AgentState::Waiting | AgentState::Unstable => {
                self.agent_state = AgentState::Running;

                let mut metadata = HashMap::<String, String>::new();
                metadata.insert("agent_id".to_string(), Value::String(self.id.clone()));
                // Call appropriate method based on agent type
                let result = match &self.agent_type {
                    AgentType::Actor(_) => self.act(source, job).await?,
                    AgentType::Reactor => self.react(source, job).await?,
                };


                // Check state
                if self.get_err_rate() <= self.accepted_err_rate {
                    self.agent_state = AgentState::Waiting;
                } else {
                    self.agent_state = AgentState::Unstable;
                }

                Ok(result)
            }
            _ => Err(anyhow::anyhow!("Can only run from Waiting or Unstable state"))
        }
    }

    pub fn stop(&mut self) -> Result<()> {
        match self.agent_state {
            AgentState::Running | AgentState::Unstable | AgentState::Waiting => {
                self.agent_state = AgentState::Stopping;
                // Cleanup logic here if needed
                self.agent_state = AgentState::Inactive;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Cannot stop from current state"))
        }
    }

    pub fn get_err_rate(&self) -> f32 {
        // Calculate error rate across all steps
        let total_runs: u64 = self.steps.iter()
            .map(|step| step.get_run_count())
            .sum();
        
        if total_runs == 0 {
            return 0.0; // No runs yet, assume ok
        }
    
        let total_failures: u64 = self.steps.iter()
            .map(|step| step.get_run_count() - step.get_success_count())
            .sum();
    
        // Calculate error rate and return it
        total_failures as f32 / total_runs as f32
    }

    pub fn check_state(&self) -> &AgentState {
        &self.agent_state
    }

}


#[async_trait]
impl CanReact for Agent {
    async fn react(&mut self, source: DataSource, job: Option<&mut Job>) -> Result<Value> {
        // Create a RuntimeSession with the agent's steps and input data
        let mut rts = RuntimeSession::new(&mut self.steps, source);
        
        // Run all steps and return the final result
        match rts.run_all(true).await {
            Ok(Some(result)) => Ok(result),
            Ok(None) => Ok(Value::Null),
            Err(e) => Err(e.into()),
        }
    }
}

#[async_trait]
impl CanAct for Agent {
    fn schedule(&self) -> &str {
        match &self.agent_type {
            AgentType::Actor(cron_schedule) => cron_schedule,
            _ => "0 0 * * * *"  // Default to running every hour if not an actor
        }
    }
    
    async fn act(&mut self, source: DataSource, job: Option<&mut Job>) -> Result<Value> {
        // Create a RuntimeSession with empty input data
        let mut rts = RuntimeSession::new(
            &mut self.steps, 
            source
        );
        
        // Run all steps and return the final result
        let result = match rts.run_all(true).await {
            Ok(Some(result)) => Ok(result),
            Ok(None) => Ok(Value::Null),
            Err(e) => Err(anyhow::anyhow!("Agent execution failed: {}", e))
        };

        // Update job status if provided
        if let Some(job) = job { job.update_status_from_result(&result); }

        result
    }
}
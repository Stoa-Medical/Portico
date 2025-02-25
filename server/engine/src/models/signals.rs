use crate::{IdFields, TimestampFields};
use crate::models::agents::Agent;
use crate::RunningStatus;
use serde_json::Value;

pub struct Signal<'a> {
    // Database fields
    identifiers: IdFields,
    timestamps: TimestampFields,
    user_requested_uuid: String,
    requested_agent: &'a Agent<'a>,
    status: RunningStatus,
    description: String,
    initial_data: Value,
    result_data: Option<Value>,
    error_message: Option<String>,
}

impl<'a> Signal<'a> {
    pub fn new(
        identifiers: IdFields,
        user_requested_uuid: String,
        requested_agent: &'a Agent<'a>,
        description: String,
        initial_data: Value,
    ) -> Self {
        Self {
            identifiers,
            timestamps: TimestampFields::new(),
            user_requested_uuid,
            requested_agent,
            status: RunningStatus::Pending,
            description,
            initial_data,
            result_data: None,
            error_message: None,
        }
    }

    pub fn process(&mut self, initial_data: &Value) -> Result<&Value, String> {
        if self.status != RunningStatus::Pending {
            return Err(format!("Cannot process signal in {:?} state", self.status));
        }
        
        self.status = RunningStatus::InProgress;
        
        // Execute the agent with the initial data
        match self.requested_agent.run(initial_data) {
            Ok(result) => {
                self.result_data = Some(result);
                self.status = RunningStatus::Completed;
                Ok(self.result_data.as_ref().unwrap())
            },
            Err(e) => {
                self.status = RunningStatus::Failed;
                self.error_message = Some(e.clone());
                Err(e)
            }
        }
    }

    pub fn abort(&mut self) -> Result<(), String> {
        if self.status == RunningStatus::Completed || 
           self.status == RunningStatus::Failed {
            return Err(format!("Cannot abort signal in {:?} state", self.status));
        }
        
        self.status = RunningStatus::Failed;
        self.error_message = Some("Signal aborted".to_string());
        Ok(())
    }

    // Methods needed for RuntimeSession integration
    pub fn set_status(&mut self, status: RunningStatus) {
        self.status = status;
    }
    
    pub fn set_error_message(&mut self, message: String) {
        self.error_message = Some(message);
    }
    
    pub fn set_result_data(&mut self, data: Value) {
        self.result_data = Some(data);
    }

    // Getters
    pub fn status(&self) -> &RunningStatus {
        &self.status
    }
    
    pub fn description(&self) -> &str {
        &self.description
    }
    
    pub fn agent(&self) -> &Agent<'a> {
        self.requested_agent
    }
    
    pub fn initial_data(&self) -> &Value {
        &self.initial_data
    }
    
    pub fn result_data(&self) -> Option<&Value> {
        self.result_data.as_ref()
    }
    
    pub fn error_message(&self) -> Option<&String> {
        self.error_message.as_ref()
    }
}

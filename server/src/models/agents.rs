// /// Class for defining Agents
// ///   An Agent is a listener that can receive + react to data
// ///   An Agent can take Steps to achieve its goal (based on heuristics)
// ///     Steps are python code, and the scaffolding is defined in `steps.rs`

// use super::steps::Step;
// use crate::{CanReceive, CanReact, DataProtocol};

// use serde_json::Value;
// use uuid::Uuid;

// /// Agents are units of action
// struct Agent {
//     /// The UUID identifier
//     id: Uuid,
//     /// What does this agent do (as described by a human)?
//     description: String,
//     /// The state of the agent
//     state: AgentState,
//     /// The steps the agent should take
//     steps: Vec<Step>,
//     /// The allowed error rate before flagging as AgentState::Unstable
//     accepted_err_rate: f32,
//     /// Outgoing destionations (if applicable)
//     outgoing_dest_list: Option<Vec<String>>,
//     /// Incoming port (if applicable)
//     incoming_port: Option<u16>
// }


// enum AgentState {
//     Waiting,
//     Active,
//     Unstable,
//     Stopping,
//     Inactive
// }


// impl Agent {
//     fn new(description: String, accepted_err_rate: f32, steps: Vec<Step>, incoming_port: Option<u16>, outgoing_dest_list: Option<Vec<String>>) -> Self {
//         Self {
//             id: Uuid::new_v4(),
//             description,
//             state: AgentState::Waiting,
//             steps,
//             accepted_err_rate,
//             incoming_port,
//             outgoing_dest_list
//         }
//     }
    
// }
 

// impl CanReact for Agent {
//     /// Pass comma-delimited string of outgoing ports
//     fn outgoing_destinations(&self) -> Vec<String> {
//         match &self.outgoing_dest_list {
//             Some(dest_list) => dest_list
//                 .iter()
//                 .map(|p| p.to_string())
//                 .collect::<Vec<String>>(),
//             None => Vec::<String>::new()
//         }
//     }
//     /// Based on the data, performs the corresponding steps and returns to the outgoing destination
//     fn react(&self, source: Value, protocol: DataProtocol) -> Result<(), std::io::Error> {
//         // TODO: Implement reaction logic
//         Err(std::io::Error::new(std::io::ErrorKind::Other, "Not implemented"))
//     }
// }

// impl CanReceive for Agent {
//     /// The port the channel can receive on (only 1 allowed)
//     fn incoming_port(&self) -> u16 {
//         self.incoming_port.unwrap_or(0)
//     }
//     /// Starts listener on the incoming port
//     fn receive(&self, protocol: DataProtocol) -> Result<Value, std::io::Error> {
//         // TODO: Implement actual network logic
//         Err(std::io::Error::new(std::io::ErrorKind::Other, "Not implemented"))
//     }
    
// }
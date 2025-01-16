#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

/// Lib module (access with `crate::`. Traits go here (stylistic choice)!

/// Module with different data models
pub mod models;

/// Supported network protocols for Agents
pub enum DataProtocol {
    // PlaintextTcpIp,  // TODO: Implement this for HL7 messages
    // XmlSoap,
    JsonRest
}

use serde_json::Value;

/// Something that can receive data on an incoming port
pub trait CanReceive {
    /// The port the channel can receive on (only 1 allowed)
    fn incoming_port(&self) -> u16;
    /// Starts listener on the incoming port
    fn receive(&self, protocol: DataProtocol) -> Result<Value, std::io::Error>;
}

/// Something that can send data to an outgoing destination
pub trait CanReact {
    /// The destination for the resulting action (expressed as a String)
    fn outgoing_destinations(&self) -> Vec<String>;
    /// Performs the action in reaction to the data, and returns to the outgoing destination
    fn react(&self, source: Value, protocol: DataProtocol) -> Result<(), std::io::Error>;
}

pub trait CanSave {

}


/// Tests
#[cfg(test)]
mod tests {
    use super::*;

    
}
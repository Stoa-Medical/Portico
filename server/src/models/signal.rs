pub enum SignalResource {
    /// Signal originated from an Agent (by Agent ID).
    Agent(String),
    /// Signal originated from a User (by User ID).
    User(String),
    /// Signal generated by the System.
    System,
    /// Signal related to a Mission (by Mission ID).
    Mission(String),
    /// Signal related to a Step execution (by Step ID).
    Step(String),
} 
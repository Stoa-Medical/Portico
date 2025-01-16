/// Module for defining and managing Agents
pub mod agents;
/// Module for defining and running Steps
pub mod steps;
/// Module for interfacing with the database
pub mod db;


/// Tests
#[cfg(test)]
use {
    serde_json::Value
};

#[cfg(test)]
struct TestContext{
    data: Value,
}
#[cfg(test)]
mod tests {
    use super::*;


}
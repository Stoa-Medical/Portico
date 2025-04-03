use std::io::{self, Read, Write};
use anyhow::{Result, anyhow};
use serde_json::Value;
use portico_shared::Agent;
use portico_shared::models::agents::AgentState;

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    // Buffer for reading the length prefix
    let mut len_buf = [0u8; 4];

    loop {
        // Read message length (4-byte prefix)
        match handle.read_exact(&mut len_buf) {
            Ok(_) => {
                // Convert bytes to u32 length
                let length = u32::from_be_bytes(len_buf) as usize;

                // Allocate buffer for the message
                let mut buffer = vec![0u8; length];

                // Read the actual message
                match handle.read_exact(&mut buffer) {
                    Ok(_) => {
                        // Process the message
                        if let Err(e) = process_message(&buffer).await {
                            eprintln!("Error processing message: {}", e);
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                        // Parent process closed the pipe
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error reading message: {}", e);
                        break;
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // Parent process closed the pipe
                break;
            }
            Err(e) => {
                eprintln!("Error reading message length: {}", e);
                break;
            }
        }
    }

    Ok(())
}

#[derive(serde::Deserialize)]
struct Message {
    agent: Agent,
    data: Value,
}

async fn process_message(data: &[u8]) -> Result<()> {
    // Parse the message
    let message: Message = serde_json::from_slice(data)?;

    // Start the agent if it's not already running
    let mut agent = message.agent;
    if agent.agent_state == AgentState::Inactive {
        agent.start()?;
    }

    // Run the agent with the provided data
    match agent.run(message.data).await {
        Ok(session) => {
            // TODO: Send-back just the JSON result
            // TODO: Think about how writing to the db should be
            // Serialize and write the result
            let result = serde_json::to_vec(&session)?;
            let len = result.len() as u32;

            // Write length prefix
            io::stdout().write_all(&len.to_be_bytes())?;
            // Write result
            io::stdout().write_all(&result)?;
            io::stdout().flush()?;

            Ok(())
        }
        Err(e) => Err(anyhow!("Agent run failed: {}", e))
    }
}

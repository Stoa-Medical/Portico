use anyhow::Result;
use serde_json::Value;
use std::net::TcpStream;
use std::io::Read;

#[derive(Debug)]
pub enum BridgeMessage {
    ServerInit(bool),
    CreateSignal(Value),
    CreateAgent(Value),
    UpdateAgent(Value),
    DeleteAgent(Value),
}

// TODO: Have this serialize to BridgeMessage
pub fn read_json_message(stream: &mut TcpStream) -> Result<BridgeMessage> {
    // Read 4-byte length prefix (u32 in network byte order)
    let mut size_buffer = [0; 4];
    stream.read_exact(&mut size_buffer)?;
    let message_size = u32::from_be_bytes(size_buffer) as usize;

    // Allocate buffer of exact size and read the whole message
    let mut buffer = vec![0; message_size];
    stream.read_exact(&mut buffer)?;
    let data = String::from_utf8_lossy(&buffer);

    match serde_json::from_str::<Value>(&data) {
        Ok(json) => {
            // `ServerInit` if it contains the `server-init` field with a bool value
            //  (For the CRUD updates below, return the `data.record` value as part of the final result)
            // `CreateSignal` if the table is "signals"
            // `CreateAgent` if the table is "agents" (`data.table`) and and type is "CREATE" (`data.type`)
            // `UpdateAgent` ... "UPDATE" (`data.type`)
            // `DeleteAgent` ... "DELETE" (`data.type`)
            // Otherwise: return Err
            if let Some(init) = json.get("server-init") {
                if let Some(init_value) = init.as_bool() {
                    Ok(BridgeMessage::ServerInit(init_value))
                } else {
                    Err(anyhow::anyhow!("server-init value must be a boolean"))
                }
            } else if let Some(data) = json.get("data") {
                let table = data.get("table")
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing or invalid table field"))?;

                match table {
                    "signals" => {
                        Ok(BridgeMessage::CreateSignal(data.get("record").cloned().unwrap_or(Value::Null)))
                    },
                    "agents" => {
                        let event_type = data.get("type")
                            .and_then(|t| t.as_str())
                            .ok_or_else(|| anyhow::anyhow!("Missing or invalid type field"))?;

                        let record = data.get("record").cloned().unwrap_or(Value::Null);

                        match event_type {
                            "INSERT" => Ok(BridgeMessage::CreateAgent(record)),
                            "UPDATE" => Ok(BridgeMessage::UpdateAgent(record)),
                            "DELETE" => Ok(BridgeMessage::DeleteAgent(record)),
                            _ => Err(anyhow::anyhow!("Unsupported event type: {}", event_type))
                        }
                    },
                    _ => Err(anyhow::anyhow!("Unsupported table: {}", table))
                }
            } else {
                Err(anyhow::anyhow!("Unrecognized message format"))
            }
        },
        Err(e) => Err(anyhow::anyhow!("Failed to parse JSON: {}", e))
    }
}

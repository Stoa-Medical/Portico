use anyhow::Result;
use serde_json::Value;
use std::net::TcpStream;
use std::io::Read;

pub fn read_json_message(stream: &mut TcpStream) -> Result<Value> {
    // Read 4-byte length prefix (u32 in network byte order)
    let mut size_buffer = [0; 4];
    stream.read_exact(&mut size_buffer)?;
    let message_size = u32::from_be_bytes(size_buffer) as usize;

    // Allocate buffer of exact size and read the whole message
    let mut buffer = vec![0; message_size];
    stream.read_exact(&mut buffer)?;
    let data = String::from_utf8_lossy(&buffer);

    match serde_json::from_str::<Value>(&data) {
        Ok(json) => Ok(json),
        Err(e) => Err(anyhow::anyhow!("Failed to parse JSON: {}", e))
    }
}

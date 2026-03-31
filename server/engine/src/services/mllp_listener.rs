use anyhow::Result;

/// Minimal Lower Layer Protocol (MLLP) listener for receiving HL7v2 messages
/// over TCP connections.
///
/// MLLP wraps HL7v2 messages with start/end block characters:
///   <VT> (0x0B) + message + <FS><CR> (0x1C 0x0D)
///
/// This is a stub implementation. Future versions will open a TCP listener,
/// parse MLLP frames, and route messages into the engine pipeline.
pub struct MLLPListener {
    pub listen_address: String,
}

impl MLLPListener {
    pub fn new(listen_address: String) -> Self {
        Self { listen_address }
    }

    /// Start the MLLP listener.
    ///
    /// Currently a stub that logs and returns Ok.
    /// Future implementation will bind a TCP socket, accept connections,
    /// and parse incoming MLLP-framed HL7v2 messages.
    pub async fn start(&self) -> Result<()> {
        println!(
            "[INFO] MLLP listener not yet implemented (configured for {})",
            self.listen_address
        );
        Ok(())
    }
}

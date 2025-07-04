fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Try different paths for the proto file to handle both local and Docker environments
    let proto_paths = [
        "../proto/bridge_message.proto",           // Local development path
        "/workspace/server/proto/bridge_message.proto", // Docker container path
        "server/proto/bridge_message.proto",       // Build context path
    ];

    // Try each path until one works
    let mut success = false;
    let mut last_error = None;

    for path in proto_paths {
        match tonic_build::compile_protos(path) {
            Ok(_) => {
                success = true;
                println!("Successfully compiled proto from: {}", path);
                break;
            }
            Err(e) => {
                println!("Failed to compile proto from {}: {}", path, e);
                last_error = Some(e);
            }
        }
    }

    if success {
        Ok(())
    } else {
        Err(Box::new(last_error.unwrap()))
    }
}

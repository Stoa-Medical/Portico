FROM rust:latest AS builder

# Install Python and protobuf compiler in builder stage
RUN apt-get update && apt-get install -y \
    python3 \
    python3-dev \
    python3-pip \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Set up the workspace structure
WORKDIR /usr/src/workspace
# Copy the shared crate first
COPY shared/ /usr/src/workspace/shared/
# Copy the proto files
COPY server/proto/ /usr/src/workspace/server/proto/
# Also copy proto files to workspace/proto for the build script
COPY server/proto/ /workspace/server/proto/
# Copy the engine crate
COPY server/engine/ /usr/src/workspace/server/engine/

# Build from the engine directory
WORKDIR /usr/src/workspace/server/engine
RUN cargo build --release

FROM debian:bookworm-slim

# Install Python in runtime stage
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/workspace/server/engine/target/release/portico-engine /usr/local/bin/

# Create a directory for any runtime files
WORKDIR /app

# Expose the port the engine listens on
EXPOSE 50051

# Run the engine
CMD ["portico-engine"]

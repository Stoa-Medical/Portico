FROM rust:1.84-slim AS builder

# Install required build dependencies including Python for PyO3 and OpenSSL
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    pkg-config \
    python3 \
    python3-dev \
    python3-pip \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set Python environment variables for PyO3 and SQLx offline mode
ENV PYTHONPATH=/usr/lib/python3/dist-packages \
    PYO3_PYTHON=/usr/bin/python3 \
    SQLX_OFFLINE=true \
    DATABASE_URL=postgresql://postgres:postgres@postgres:5432/postgres

# Create the workspace structure
WORKDIR /app

# Copy the entire shared crate first - including its full structure
COPY shared/ /app/shared/
# Copy proto files
COPY server/proto/ /app/server/proto/
# Copy the engine crate
COPY server/engine/ /app/server/engine/

# Build the application
WORKDIR /app/server/engine
RUN cargo build --release

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    netcat-traditional \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/server/engine/target/release/portico-engine /usr/local/bin/

# Set appropriate permissions
RUN chmod +x /usr/local/bin/portico-engine

# Create directory for any runtime files
WORKDIR /app

# Expose port for gRPC
EXPOSE 50051

# Set default environment variables
ENV LLM_API_ENDPOINT="https://api.together.xyz/v1/chat/completions" \
    GRPC_PORT=50051

# Set healthcheck
HEALTHCHECK --interval=5s --timeout=5s --start-period=5s --retries=3 \
    CMD nc -z localhost 50051 || exit 1

# Run the engine
CMD ["portico-engine"]

FROM rust:1.84-slim AS builder

# Install required build dependencies (no Python/PyO3 needed in v2)
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set SQLx offline mode
ENV SQLX_OFFLINE=true \
    DATABASE_URL=postgresql://postgres:postgres@postgres:5432/postgres

# Create the workspace structure
WORKDIR /app

# Copy the entire database crate first
COPY server/database/ /app/server/database/
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
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/server/engine/target/release/portico-engine /usr/local/bin/

# Set appropriate permissions
RUN chmod +x /usr/local/bin/portico-engine

# Create directory for any runtime files
WORKDIR /app

# Expose ports for gRPC and health check
EXPOSE 50051
EXPOSE 8080

# Set default environment variables
ENV GRPC_PORT=50051 \
    HEALTH_PORT=8080

# Health check via HTTP endpoint
HEALTHCHECK --interval=10s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -sf http://localhost:8080/health || exit 1

# Run the engine
CMD ["portico-engine"]

FROM rust:latest AS builder

# Install Python in builder stage
RUN apt-get update && apt-get install -y \
    python3 \
    python3-dev \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

# Install Python in runtime stage
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/portico-engine /usr/local/bin/

# Create a directory for any runtime files
WORKDIR /app

# Expose the port the engine listens on
EXPOSE 50051

# Run the engine
CMD ["portico-engine"]

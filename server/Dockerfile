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

# Install Python in runtime stage (using apt instead of building from source)
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# TODO: Add Atlas tool to run migrations

# Copy the Rust binary from builder
COPY --from=builder /usr/src/app/target/release/portico /usr/local/bin/
CMD ["portico"]
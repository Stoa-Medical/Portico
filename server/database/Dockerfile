FROM rust:1.84-slim

# Install required dependencies
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    pkg-config \
    python3 \
    python3-dev \
    python3-pip \
    libssl-dev \
    netcat-traditional \
    && rm -rf /var/lib/apt/lists/*

# Install SQLx CLI
RUN cargo install sqlx-cli --no-default-features --features postgres

# Create the workspace structure
WORKDIR /app

# Copy the shared crate
COPY shared/ /app/shared/

# Copy schema for migrations
COPY server/scheme.hcl /app/server/scheme.hcl

# Create a script to prepare SQLx files
RUN echo '#!/bin/bash \n\
set -e \n\
# Wait for postgres \n\
until nc -z $POSTGRES_HOST $POSTGRES_PORT; do \n\
    echo "Waiting for postgres..." \n\
    sleep 1 \n\
done \n\
echo "Postgres is up, preparing SQLx query files..." \n\
cd /app/shared \n\
SQLX_OFFLINE=false cargo sqlx prepare -- --tests \n\
echo "SQLx preparation complete" \n\
' > /app/prepare.sh && chmod +x /app/prepare.sh

# Set the command to run the preparation script
CMD ["/app/prepare.sh"]

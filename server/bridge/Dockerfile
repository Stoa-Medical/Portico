# Use Python 3.12 or higher
FROM python:3.12-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    netcat-traditional \
    && rm -rf /var/lib/apt/lists/*

# Install Python tools
RUN pip install --no-cache-dir uv==0.1.23

# Set environment variables
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    UV_SYSTEM_PYTHON=true

# Create workspace structure
WORKDIR /app

# Copy requirements first for better layer caching
COPY server/bridge/pyproject.toml /app/server/bridge/

# Copy proto files
COPY server/proto/ /app/server/proto/

# Set working directory to bridge
WORKDIR /app/server/bridge

# Install dependencies
RUN --mount=type=cache,target=/root/.cache \
    uv pip install -e . --no-cache-dir

# Copy source code
COPY server/bridge/src/ /app/server/bridge/src/

# Generate protobuf files
RUN python -m src.proto.build_proto

# Default environment variables
ENV POSTGRES_USER=postgres \
    POSTGRES_PASSWORD=postgres \
    POSTGRES_HOST=postgres \
    POSTGRES_PORT=5432 \
    POSTGRES_DB=postgres \
    ENGINE_HOST=engine \
    ENGINE_PORT=50051

# Add healthcheck
HEALTHCHECK --interval=5s --timeout=5s --start-period=5s --retries=3 \
    CMD python -c "import socket; s = socket.socket(); s.connect(('localhost', 50052))" || exit 1

# Expose port for service
EXPOSE 50052

# Ensure engine is running before starting bridge
CMD ["sh", "-c", "until nc -z $ENGINE_HOST $ENGINE_PORT; do echo 'Waiting for engine...'; sleep 1; done && python -m src.main"]

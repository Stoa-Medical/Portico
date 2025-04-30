# Use Python 3.9 or higher
FROM python:3.13-slim

WORKDIR /app

# Install uv and grpcio-tools
RUN pip install --no-cache-dir uv grpcio-tools

# Set environment variables
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    UV_SYSTEM_PYTHON=1

# Create workspace structure
RUN mkdir -p /workspace/server/bridge /workspace/server/proto

# Copy proto files first
COPY server/proto/ /workspace/server/proto/

# Copy bridge service files
COPY server/bridge/pyproject.toml /workspace/server/bridge/
COPY server/bridge/uv.lock* /workspace/server/bridge/
COPY server/bridge/src/ /workspace/server/bridge/src/

# Set working directory to bridge service
WORKDIR /workspace/server/bridge

# Create virtual environment and install dependencies
RUN --mount=type=cache,target=/root/.cache \
    uv venv /workspace/server/bridge/.venv && \
    # Install all dependencies directly from pyproject.toml
    uv pip install -e . --no-cache-dir \
    && echo "source /workspace/server/bridge/.venv/bin/activate" >> ~/.bashrc

# Generate gRPC code
RUN . /workspace/server/bridge/.venv/bin/activate && \
    python -m src.proto.build_proto

# Add the virtual environment to PATH
ENV PATH="/workspace/server/bridge/.venv/bin:$PATH"

# Expose port for gRPC communication with engine
EXPOSE 50052

# Start the service
CMD ["python", "-m", "src.main"]

# Use Python 3.9 or higher
FROM python:3.10-slim

WORKDIR /app

# Install uv
RUN pip install --no-cache-dir uv

# Copy only what's needed for dependency installation
COPY pyproject.toml .
COPY uv.lock* ./

# Set environment variables
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    UV_SYSTEM_PYTHON=1

# Create virtual environment and install dependencies
RUN --mount=type=cache,target=/root/.cache \
    uv venv /app/.venv && \
    # Install all dependencies directly from pyproject.toml
    uv pip install -e . --no-cache-dir \
    && echo "source /app/.venv/bin/activate" >> ~/.bashrc

# Copy source code
COPY src/ ./src/

# Generate gRPC code
RUN . /app/.venv/bin/activate && \
    python -m src.proto.build_proto

# Add the virtual environment to PATH
ENV PATH="/app/.venv/bin:$PATH"

# Start the service
CMD ["python", "-m", "src.main"]

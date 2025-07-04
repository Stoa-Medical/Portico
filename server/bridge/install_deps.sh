#!/bin/bash
# Script to set up development environment

# Ensure uv is installed
if ! command -v uv &> /dev/null; then
    echo "uv is not installed. Installing..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
else
    echo "Updating uv to the latest version..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
fi

# Create virtual environment if it doesn't exist
if [ ! -d ".venv" ]; then
    echo "Creating virtual environment..."
    uv venv
fi

# Activate virtual environment
echo "Activating virtual environment..."
source .venv/bin/activate || source ./.venv/bin/activate

# Install all dependencies including dev dependencies
echo "Installing dependencies..."
uv pip install -e ".[dev]"

# Generate proto files
echo "Generating gRPC code..."
python -m src.proto.build_proto

# Generate lock file
echo "Generating lock file..."
uv pip freeze > uv.lock

echo "Done! Environment is ready."
echo "To activate the environment, run: source .venv/bin/activate"
echo ""
echo "To run the service, make sure the engine service is running, then:"
echo "python -m src.main"

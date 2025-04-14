# PowerShell script to set up development environment

# Ensure uv is installed
if (-not (Get-Command uv -ErrorAction SilentlyContinue)) {
    Write-Host "uv is not installed. Installing..."
    irm 'https://astral.sh/uv/install.ps1' | iex
} else {
    Write-Host "Updating uv to the latest version..."
    irm 'https://astral.sh/uv/install.ps1' | iex
}

# Create virtual environment if it doesn't exist
if (-not (Test-Path ".venv")) {
    Write-Host "Creating virtual environment..."
    uv venv
}

# Activate virtual environment
Write-Host "Activating virtual environment..."
& .\.venv\Scripts\Activate.ps1

# Install all dependencies including dev dependencies
Write-Host "Installing dependencies..."
uv pip install -e ".[dev]"

# Generate proto files
Write-Host "Generating gRPC code..."
python -m src.proto.build_proto

# Generate lock file
Write-Host "Generating lock file..."
uv pip freeze > uv.lock

Write-Host "Done! Environment is ready."
Write-Host "To activate the environment, run: .\.venv\Scripts\Activate.ps1"
Write-Host ""
Write-Host "To run the service, make sure the engine service is running, then:"
Write-Host "python -m src.main"

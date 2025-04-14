# Portico Bridge

A Python microservice that bridges between Supabase and the Portico Engine using gRPC.

## Setup with uv

This project uses [uv](https://github.com/astral-sh/uv) for dependency management.

### Quick Setup

For a quick setup, use the provided installation script:

```bash
# Unix/macOS
chmod +x install_deps.sh
./install_deps.sh

# Windows (PowerShell)
.\install_deps.ps1
```

### Manual Installation

1. Install uv:

```bash
# macOS/Linux
curl -LsSf https://astral.sh/uv/install.sh | sh

# Windows PowerShell
irm 'https://astral.sh/uv/install.ps1' | iex
```

2. Create a virtual environment and install dependencies:

```bash
# Create a virtual environment
uv venv

# Activate the virtual environment (unix/macOS)
source .venv/bin/activate

# Activate the virtual environment (Windows)
.venv\Scripts\activate

# Install dependencies (if you have uv.lock)
uv pip sync

# Or install directly from pyproject.toml
uv pip install -e .
```

If you need to update dependencies:

```bash
# Install dependencies based on pyproject.toml
uv pip install -e .

# Generate/update lock file
uv pip freeze > uv.lock
```

### Development

Generate gRPC code:

```bash
python -m src.proto.build_proto
```

Run the service:

```bash
python -m src.main
```

### Docker

Build the Docker image:

```bash
docker build -t portico-bridge .
```

Run the Docker container:

```bash
docker run -p 50051:50051 --env-file .env portico-bridge
```

### Troubleshooting

If you encounter missing dependencies, try:

```bash
# Install all dependencies including optional ones
uv pip install -e ".[dev]"
```

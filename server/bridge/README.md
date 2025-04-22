# Portico Bridge Service

The Bridge Service is a middleware component that connects the Supabase database to the Portico Engine. It listens to Supabase Realtime events, transforms them into gRPC messages, and forwards them to the Engine service.

## Architecture

```
┌────────┐         ┌──────────┐         ┌────────┐         ┌────────┐
│  User  │────────▶│`supabase`│────────▶│`bridge`│────────▶│`engine`│
└────────┘         └──────────┘         └────────┘         └────────┘
                       ▲                                      │
                       └──────────────────────────────────────┘
```

## Signal Types

The Bridge Service handles three types of signals:

1. **Command Signals**: Create, update, delete, or run operations on Agents and Steps
2. **Sync Signals**: Re-read and serialize specific entities or all entities
3. **FYI Signals**: Informational updates (e.g., general state changes) -- these are not acted on

## Setup with uv

This project uses [uv](https://github.com/astral-sh/uv), a fast Python package installer and resolver written in Rust.

### Installation

First, install `uv`:

```bash
# macOS/Linux
curl -LsSf https://astral.sh/uv/install.sh | sh

# Windows PowerShell
irm 'https://astral.sh/uv/install.ps1' | iex
```

### Setting Up the Environment

Create and activate a virtual environment:

```bash
# Create a virtual environment
uv venv

# Activate the virtual environment (Unix/macOS)
source .venv/bin/activate

# Activate the virtual environment (Windows)
.venv\Scripts\activate
```

Install dependencies from `pyproject.toml`:

```bash
# Install all dependencies
uv pip install -e .

# Or sync dependencies with existing lock file
uv pip sync
```

### Configure the Environment

Copy the example environment file:

```bash
cp .env-example .env
```

Update variables in `.env` with your Supabase URL, key, and Engine service details.

## Running the Service

Run the service directly:

```bash
python -m src.main
```

### Using Docker

Build and run with Docker:

```bash
docker build -t portico-bridge .
docker run -p 50051:50051 --env-file .env portico-bridge
```

## Development

### Managing Dependencies

Add new dependencies:

```bash
# Add a dependency
uv add <package-name>

# Add a development dependency
uv add --dev <package-name>
```

Generate a lock file for reproducible builds:

```bash
uv pip freeze > uv.lock
```

### Proto Generation

The service automatically generates Python protobuf code on startup. To manually generate it:

```bash
python -m src.proto.build_proto
```

### Testing

Run the tests using pytest:

```bash
# Run all tests
python -m pytest

# Run with coverage
python -m pytest --cov=src
```

To test the bridge service without a running Engine:

1. Start the mock engine service:
   ```bash
   python -m tests.mock_engine
   ```

2. In another terminal, start the bridge service:
   ```bash
   python -m src.main
   ```

3. Send test signals using the test client:
   ```bash
   python -m tests.test_client command
   python -m tests.test_client sync
   python -m tests.test_client fyi
   ```

## Signal Flow

1. User creates a Signal in Supabase
2. Bridge receives the Supabase Realtime event
3. Bridge transforms the event into a gRPC SignalRequest
4. Engine processes the request and returns a SignalResponse
5. Bridge updates the Signal in Supabase with the response data

## Signal Request Structure

```proto
message SignalRequest {
  string global_uuid = 1;
  string user_requested_uuid = 2;
  SignalType signal_type = 3;

  // Optional payload based on signal type
  oneof payload {
    CommandPayload command = 4;
    SyncPayload sync = 5;
    google.protobuf.Struct fyi_data = 6;
  }
}
```

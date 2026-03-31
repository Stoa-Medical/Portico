# Server

Backend services for Portico — containerized and deployed alongside the Next.js web app.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Docker Compose (local) / Container platform (production)   │
│                                                             │
│  ┌──────────────────────┐    ┌─────────────────────────┐   │
│  │   Rust Engine         │    │   Python Sidecar         │   │
│  │   (gRPC :50051)       │◄──►│   (HTTP :8001)           │   │
│  │   (health :8080)      │    │                          │   │
│  │                        │    │  Sandboxed step exec     │   │
│  │  Signal routing        │    │  Any Python version      │   │
│  │  Workflow orchestration│    │  Crash-isolated          │   │
│  │  HL7v2/X12 parsing     │    └─────────────────────────┘   │
│  │  FHIR construction     │                                  │
│  └───────┬────────────────┘                                  │
│          │                                                   │
└──────────┼───────────────────────────────────────────────────┘
           │
     ┌─────┼──────────────────────┐
     │     │                      │
     ▼     ▼                      ▼
  Neon    Medplum             AI Gateway
  Postgres (FHIR R4)         (LLM steps)
```

## Components

### Rust Engine (`engine/`)

Workflow orchestrator — the core of Portico's signal processing pipeline.

- Receives signals via gRPC (`run`, `sync`, `fyi`) from the Next.js web app
- Consumes async signals from Upstash Redis Streams
- Orchestrates agent execution with per-agent tokio MPSC queues
- Delegates: LLM steps → AI Gateway, FHIR ops → Medplum REST, Python steps → sidecar
- Persists runtime sessions and results to Neon Postgres via SQLx

### Python Sidecar (`python-sidecar/`)

Sandboxed Python step execution (replaces the v1 Python bridge).

- Receives step code + input JSON from the Rust engine
- Executes in an isolated namespace with configurable timeout (`MAX_EXECUTION_TIME`)
- Crash isolation: a Python failure does not take down the engine
- Independent Python version and dependencies from the Rust build

### Database Crate (`database/`)

Shared Rust library for database models, operations, and schema.

- Models: Agent, Step, Signal, RuntimeSession, Connection
- Database operations via SQLx
- Atlas HCL schema definition (`scheme.hcl`)

### Protobuf (`proto/`)

Shared gRPC/Protobuf definitions:
- `bridge_message.proto` — signal processing RPCs
- `python_step.proto` — Python sidecar communication

## Quick Start

### Docker Compose (recommended)

```bash
cd server
docker compose up --build
```

This launches:
- **Postgres** on `:54322` (local dev — production uses Neon)
- **Schema migration** via Atlas (auto-applies `scheme.hcl`)
- **Rust Engine** on `:50051` (gRPC) + `:8080` (health)
- **Python Sidecar** on `:8001`

### tmuxinator (development)

```bash
cd server
tmuxinator start
```

### Manual Setup

1. **Postgres**: `docker compose up postgres` or use Supabase CLI (`cd database && supabase start`)
2. **Engine**: `cd engine && cp .env-example .env && cargo run`
3. **Python Sidecar**: `cd python-sidecar && pip install -r requirements.txt && python server.py`

## Communication with the Web App

| Pattern | Transport | Use Case |
|---------|-----------|----------|
| **Sync signals** | gRPC (`:50051`) | Requests needing a response (workflow triggers, status queries) |
| **Async signals** | Upstash Redis Streams | Fire-and-forget, high-volume events (webhook ingestion, Medplum subscriptions) |

The web app's `lib/engine/client.ts` handles gRPC communication. Async signals are published to Redis from Next.js API routes and consumed by the engine at its own pace.

## Environment Variables

| Variable | Service | Description |
|----------|---------|-------------|
| `DATABASE_URL` | Engine | Postgres connection string |
| `GRPC_PORT` | Engine | gRPC listen port (default: 50051) |
| `HEALTH_PORT` | Engine | Health check port (default: 8080) |
| `PYTHON_SIDECAR_URL` | Engine | Sidecar endpoint (default: `http://python-sidecar:8001`) |
| `AI_GATEWAY_URL` | Engine | AI Gateway endpoint |
| `AI_GATEWAY_API_KEY` | Engine | AI Gateway auth |
| `MEDPLUM_BASE_URL` | Engine | Medplum FHIR server URL |
| `MEDPLUM_CLIENT_ID` | Engine | Medplum client credentials |
| `MEDPLUM_CLIENT_SECRET` | Engine | Medplum client credentials |
| `REDIS_URL` | Engine | Upstash Redis for async signals |
| `MAX_EXECUTION_TIME` | Sidecar | Python step timeout in seconds (default: 60) |

## Testing

```bash
cd engine && cargo test       # Rust engine tests
cd database && cargo test     # Database crate tests
```

## Structure

```
server/
├── engine/                 # Rust gRPC workflow orchestrator
│   ├── src/
│   │   ├── main.rs         #   Entry point
│   │   ├── core/           #   Workflow management, signal routing
│   │   ├── handlers/       #   Signal handlers (run, sync, fyi)
│   │   ├── services/       #   Agent management, caching
│   │   └── steps/          #   Step implementations
│   ├── Cargo.toml
│   └── Dockerfile
├── python-sidecar/         # Sandboxed Python execution
│   ├── server.py           #   HTTP server
│   ├── executor.py         #   Step runner
│   ├── requirements.txt
│   └── Dockerfile
├── database/               # Shared Rust DB crate
│   ├── src/
│   │   ├── models/         #   Agent, Step, Signal, RuntimeSession
│   │   └── tests/
│   ├── scheme.hcl          #   Atlas schema definition
│   └── Cargo.toml
├── proto/                  # Protobuf definitions
│   ├── bridge_message.proto
│   └── python_step.proto
├── docker-compose.yml      # Full local dev environment
└── .tmuxinator.yml         # tmux dev session config
```

PORTICO SERVER
==============

The Portico server is a microservices architecture comprising:
- PostgreSQL Database (via Supabase) - Core data storage
- Rust Database Crate (shared library) - Database models and operations
- Rust Engine - Processes signals and manages workflows via gRPC
- Python Bridge - Connects database events to the engine

Communication flow: Frontend ↔ Database ← Bridge → Engine

## QUICK START

### Option 1: Everything with Docker Compose
From server/ directory:

  docker compose up --build

This launches:
- PostgreSQL database (port 54322)
- Schema migration with Atlas
- Engine service (gRPC on port 50051)
- Bridge service (connects to engine)

Use Ctrl-C to stop all services.

### Option 2: Development with tmuxinator
From server/ directory:

  tmuxinator start

Launches a 3-pane tmux session:
- Pane 1: Supabase (database/supabase start)
- Pane 2: Engine (engine/cargo run)
- Pane 3: Bridge (bridge/python -m src.main)

Detach: Ctrl-b d
Re-attach: tmux attach -t portico-server

## MANUAL COMPONENT SETUP

### 1. Database Setup
  cd database
  supabase start                    # Start local Supabase
  ../reset_db.sh                    # Apply schema & seed data

Database will be available at:
- API: http://localhost:54321
- Direct connection: postgresql://postgres:postgres@localhost:54322/postgres
- Studio: http://localhost:54323

### 2. Rust Engine
  cd engine
  cp .env-example .env              # Configure DATABASE_URL
  cargo check                       # Verify compilation
  cargo run                         # Start gRPC server on :50051

### 3. Python Bridge
  cd bridge
  ./install_deps.sh                 # Setup uv environment
  source .venv/bin/activate
  cp .env-example .env              # Configure DATABASE_URL, ENGINE_*
  python -m src.main                # Start bridge service

## ARCHITECTURE OVERVIEW

### Database Crate (/database/)
Shared Rust library containing:
- Database models (Agent, Workflow, Step, Signal, RuntimeSession)
- Database operations via sqlx
- Type definitions and validation
- Agent-workflow system implementation

Key traits:
- DatabaseItem: CRUD operations for all entities
- JsonLike: JSON serialization/deserialization

### Engine (/engine/)
Rust gRPC server that:
- Receives signals via gRPC (run, sync, fyi)
- Manages workflow execution with step loading
- Coordinates agent planning and validation
- Persists runtime sessions to database

Key components:
- WorkflowManager: Message queuing and processing
- AgentManager: Agent-driven workflow composition
- Signal handlers: run, sync, fyi operations

### Bridge (/bridge/)
Python service that:
- Monitors database changes via Supabase realtime
- Forwards events to Engine via gRPC
- Handles connection resilience and retries

### Shared Database Schema (/scheme.hcl)
Atlas HCL schema defining:
- Core entities: workflows, steps, signals, runtime_sessions, agents
- Relationships and constraints
- Enums: workflow_state, step_type, signal_type, running_status

## DEVELOPMENT WORKFLOWS

### Testing
Engine (Rust):
  cd engine && cargo test

Bridge (Python):
  cd bridge && source .venv/bin/activate && pytest

Database (Rust):
  cd database && cargo test

### Database Operations
Reset schema and data:
  ./reset_db.sh

Connect to database:
  psql postgresql://postgres:postgres@localhost:54322/postgres

View logs:
  docker compose logs -f postgres

### Agent-Workflow System
The system implements agent-driven workflow composition:

1. Agents analyze objectives and create workflow specifications
2. Workflows execute steps using the runtime engine
3. Results are persisted and can trigger further workflows

Key files:
- database/src/models/agents/planner.rs - Workflow planning logic
- database/src/models/workflows/runtime.rs - Workflow execution
- engine/src/services/agent_manager.rs - Agent coordination

## REPOSITORY STRUCTURE

/server/
├── database/          # Shared Rust database library
│   ├── src/models/    # Database entities and operations
│   ├── scheme.hcl     # Atlas database schema
│   └── tests/         # Database integration tests
├── engine/            # Rust gRPC workflow engine
│   ├── src/core/      # Core workflow management
│   ├── src/handlers/  # Signal handlers (run, sync, fyi)
│   └── src/services/  # Agent management, planning, caching
├── bridge/            # Python database-to-engine bridge
│   ├── src/           # Bridge service implementation
│   └── tests/         # Bridge service tests
├── proto/             # gRPC protocol definitions
├── docker-compose.yml # Complete development environment
└── .tmuxinator.yml    # Development session configuration

## DEPENDENCIES

### System Requirements
- Docker & Docker Compose
- PostgreSQL client (psql)
- Python 3.10+ with uv package manager
- Rust with Cargo
- Supabase CLI
- Atlas CLI (https://atlasgo.io)

### Language Dependencies
- Rust: sqlx, tokio, tonic, serde, anyhow, uuid, chrono
- Python: asyncio, grpcio, supabase, psycopg2
- Database: PostgreSQL 15+

### Environment Variables
- DATABASE_URL: PostgreSQL connection string
- ENGINE_HOST/ENGINE_PORT: Engine gRPC endpoint
- SUPABASE_URL/SUPABASE_KEY: Supabase configuration

## PERFORMANCE NOTES

The system is designed for:
- Concurrent workflow execution
- Efficient step loading and caching
- Agent-driven intelligent automation
- Real-time event processing via database triggers

For production deployment, consider:
- Connection pooling configuration
- Agent garbage collection policies
- Workflow complexity limits
- Resource monitoring and alerting

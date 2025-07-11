This is the server code: Supabase configuration, a Python bridge service that forwards Supabase Realtime events, and a Rust gRPC engine that executes workflow steps.

## Quick Start (tmuxinator)

From inside `server/`, run:

```bash
tmuxinator start
```

This command launches a tmux session (defined in `.tmuxinator.yml`) with three panes:

- Supabase (pane 1) – `supabase start`
- Engine (pane 2)   – `cargo run` inside `engine/`
- Bridge (pane 3)   – `python -m src.main` inside `bridge/` (after the engine is listening on port 50051)

Detach with `Ctrl-b d` and re-attach anytime with `tmux attach -t portico-server`.

## Quick Start (Docker Compose)

From the repository root (or inside `server/`), run:

```bash
# build & start Supabase, Bridge, and Engine
docker compose up --build
```

This command launches:

- Supabase (API: 54321, DB: 54322, Studio: 54323)
- Bridge service (Python, port 50051 inside network)
- Engine service (Rust, port 50051 inside network)

Use `Ctrl-C` to stop all services. For granular control continue with the sections below.

To run components independently:

1. Supabase (local dev only)
   - cd supabase
   - Review `config.toml` (API: 54321, DB: 54322, Studio: 54323)
   - Start: `supabase start`
   - Reset schema & seed data: `../reset_db.sh`

2. Python Bridge
   - cd bridge
   - Install uv & create venv: `uv venv`
   - Activate venv: `source .venv/bin/activate`
   - Install deps: `uv pip install -e .`
   - Copy & configure env: `cp .env-example .env` (set SUPABASE_URL, SUPABASE_KEY, ENGINE_URL)
   - Run: `python -m src.main`

3. Rust Engine
   - cd engine
   - Copy & configure env: `cp .env-example .env` (set DATABASE_URL)
   - Check & build: `cargo check` or `cargo build`
   - Run: `cargo run`

REPO STRUCTURE
- `supabase/` – Supabase local instance config
- `bridge/`   – Python middleware forwarding Realtime events to engine
- `engine/`   – Rust gRPC server executing multi-step workflows
- `proto/`    – shared protobuf definitions
- `scheme.hcl`– database schema (source of truth)
- `examples/` – SQL scripts for seeding and test scenarios
- `reset_db.sh`– reset schema & seed script (uses supabase db reset & atlas)
- `docker-compose.yml`– Compose file for bridge & engine

TESTS
- Engine (Rust)
  - Unit: `engine/src/lib.rs` & submodules
  - Integration: `engine/tests/`
  - Run: `cargo test`
- Bridge (Python)
  - Tests: `bridge/tests/`
  - Run: `python -m pytest`

LOCAL TOOLKIT
- Docker & Docker Compose
- supabase CLI
- Python >=3.10 & uv (https://astral.sh/uv)
- Rust & Cargo (`cargo install cargo-audit --features=fix`)
- Atlas CLI (https://atlasgo.io)
- psql (PostgreSQL client)

DEPENDENCIES
- Python packages: see `bridge/pyproject.toml`
- Rust crates: see `engine/Cargo.toml`
- Supabase SDK, grpcio, protobuf, etc.

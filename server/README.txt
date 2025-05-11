This is the server code: Supabase configuration, a Python bridge service that forwards Supabase Realtime events, and a Rust gRPC engine that executes workflow steps.

To run full stack:
- `docker compose up`
- OR `supabase start` + `tmuxinator start .`

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

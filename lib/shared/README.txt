This is the shared crate containing core Rust data models, utilities, and logic for both the Engine and future Tauri app.

Requirements:
- Rust >=1.60 and Cargo
- SQLx CLI (`cargo install sqlx-cli`)
- Python >=3.x for pyo3 support
- PostgreSQL database for SQLx queries

Setup:
1. Copy env example:
   - `cp .env-example .env` (set `DATABASE_URL`)
2. Prepare SQLx for compile-time checks:
   - `cargo sqlx prepare -- --lib`
3. Build the crate:
   - `cargo build`
4. (Optional) Generate docs:
   - `cargo doc --open`

Modules:
- `models/` – Data models: `Agent`, `Step`, `Signal`, `RuntimeSession`
- `webscrape` – Web scraping utilities
- `PythonRuntime` – Embedded Python execution manager
- `call_llm` – LLM API helpers with retry/backoff
- Shared utilities: `load_agent_steps`, SQL fragment generators, etc.

Testing:
- Run tests:
   - `cargo test`
- For SQLx integration tests, ensure `.env` is configured and the database is running.

SQLx Offline Mode:
- The Docker build uses `SQLX_OFFLINE=true` to avoid requiring a database connection during builds
- IMPORTANT: After any changes to SQL queries or database schema, you MUST run:
  ```
  cargo sqlx prepare --workspace
  ```
- This generates/updates the sqlx-data.json files which should be committed to version control
- Failing to update these files will cause Docker builds to fail with "no cached data for this query" errors

Developer notes:
- Optional crate features in `Cargo.toml`: `strum`, `typed-builder`, `thiserror`

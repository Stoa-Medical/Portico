This is the server code.

To check compilation: `cargo check` or `cargo build`
To run with database: `docker compose up`

REPO STRUCTURE
- `engine` is the core server (Rust)
- `supabase` is config for Supabase instance
- `bridge` is lightweight middle service for handling Supabase stuff, e.g.:
    1. Forwarding Supabase Realtime changes
    2. Handling Supabase auth

TESTS
- Unit tests are either in:
    - `src/lib.rs`
    - or `src/*/mod.rs` (for submodules)
- Integration tests are in `tests/`
- Run tests with `cargo test`

The database schema is maintained in `scheme.hcl`. This file should be considered the source-of-truth for the schema

The `seed.sql` file contains sample data for development and testing purposes (NOT prod). This data includes:
- Sample agents with different states and types
- Steps associated with each agent
- Signals with various statuses
- Runtime sessions in different states

Local toolkit:
- brew (MacOS)
- cargo (Rust)
    - `cargo install cargo-audit --features=fix`

Dependencies:
- supabase
- Atlas (https://atlasgo.io)

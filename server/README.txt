This is the server code.

To check compilation: `cargo check` or `cargo build`
To run with database: `docker compose up`

REPO STRUCTURE
- `engine` is the core server runtime (Rust)
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


Local toolkit:
- brew (MacOS)

Dependencies:
- supabase
- Atlas (https://atlasgo.io)
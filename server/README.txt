This is the server code.

To check compilation: `cargo check` or `cargo build`
To run with database: `docker compose up`

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
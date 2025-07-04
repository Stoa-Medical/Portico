# Development Prerequisites

| Tool      | Version  | Notes |
|-----------|----------|-------|
| Node      | ≥ 20.x   | Use Volta or `nvm install 20` |
| pnpm      | ≥ 8.x    | `corepack enable` installs automatically |
| Rust      | stable   | `rustup toolchain install stable` (MSRV 1.75) |
| Python    | 3.11     | Required by `server/bridge` |
| Docker    | ≥ 24     | Brings up Supabase + Engine + Bridge via Compose |
| supabase-cli | optional | For local DB inspection & migrations |

> Tip: run `rustup component add clippy rustfmt` for a smoother Rust DX.

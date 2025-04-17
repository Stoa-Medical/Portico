# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands by Directory

### App (Tauri 2.0 + Svelte 5)

- Dev: `cd app && npm run dev`
- Build: `cd app && npm run build`
- Test all: `cd app && npm run test`
- Test single: `cd app && npm run test -- src/path/to/test.ts`
- TypeScript check: `cd app && npm run check`
- Tauri dev: `cd app && npm run tauri dev`

### Shared (Rust)

- Test all: `cd shared && cargo test`
- Test single: `cd shared && cargo test test_name`
- Build: `cd shared && cargo build`

### Server

- Engine (Rust): `cd server/engine && cargo test`
- Bridge (Python): `cd server/bridge && python -m pytest`
- Type check: `cd server/bridge && mypy src`
- Run all services: `cd server && docker-compose up`

## Code Style by Technology

- **Svelte/TS**: 2-space indent, strict types, PascalCase components
- **Rust**: Standard formatting, `anyhow` for errors, snake_case
- **Python**: Type annotations, mypy validation, PEP 8 formatting
- **Tests**: Component tests use `.test.ts`, integration tests at module level

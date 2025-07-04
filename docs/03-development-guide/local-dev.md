# Local Development

The fastest way to get everything running locally is to use **Docker Compose** for the backend stack and run the UI separately.

## 1. Start backend stack

```bash
cd server
docker compose up --build
```

Ports:
* Postgres: `localhost:54322`
* gRPC Engine: `localhost:50051`

> The `shared_prepare` container pre-builds SQLx query metadata so the Engine compiles faster.

## 2. Bridge hot-reload (optional)

The Bridge runs inside Docker by default, but if you want live-reload while hacking on Python code:

```bash
cd server/bridge
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
python src/main.py  # Ctrl+C to stop
```

Make sure `.env` in `server/bridge/` points to the Engine host/port.

## 3. Frontend / Desktop App

```bash
cd app
pnpm install

# Web dev server
pnpm dev

# OR – desktop app with Tauri
pnpm tauri dev
```

The web server will auto-proxy requests to Supabase; configure the project URL via `src/lib/supabase.ts`.

## Running tests

```bash
# Rust
cargo test --workspace

# Python
pytest server/bridge/tests

# Frontend
pnpm test
```

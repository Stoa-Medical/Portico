<img
  src="assets/Logo-Inline-Portico.svg"
  alt="Portico Logo"
  style="height: 56px; width: auto; max-width: 100%;"
/>

Portico is an **agentic integration engine**: an event-driven backend (Signals → Agents → Steps) with a desktop app for configuration, built for healthcare-style integration workflows.

**Project status**: MVP/prototype. I plan to rewrite this repo later; this version focuses on proving the core architecture and developer experience.

### Project Highlights

- **Desktop configuration UI**: a Svelte 5 + SvelteKit + Tauri app with Supabase auth/client integration and a CodeMirror-based Python editor UI.
- **Event-driven workflow engine**: a Rust gRPC server processes `run` / `sync` / `fyi` signals and persists `RuntimeSession` results to Postgres.
- **Clean service boundary**: a Python “bridge” listens to **Supabase Realtime** (Postgres changes) and forwards events to the Rust engine over **gRPC/Protobuf**.
- **Schema-as-code**: database schema is defined in **Atlas** HCL (`server/database/scheme.hcl`) and applied in dev with `atlas schema apply`.
- **Safety-oriented step model**: step capabilities are validated (tool gating), and the `webscrape` step includes a robots.txt check in the shared DB crate.

If you want the deeper product/architecture intent, see `design/1-mvp.md`.

### Repository structure

- **`app/`**: Tauri 2.0 desktop app (SvelteKit + Svelte 5 + TypeScript + Tailwind + Flowbite-Svelte).
- **`server/engine/`**: Rust gRPC engine (signal handlers, workflow execution, caching/monitoring utilities).
- **`server/bridge/`**: Python bridge service (Supabase Realtime → gRPC).
- **`server/database/`**: shared Rust crate for models + validation + SQLx queries + utilities (incl. `webscrape`).
- **`server/proto/`**: Protobuf definitions shared across services.

### Architecture at a glance

```mermaid
graph TB
    subgraph "Desktop App"
        A[SvelteKit (Svelte 5)]
        B[Tauri (Rust)]
        A <-->|IPC| B
    end

    subgraph "Server"
        C[(Postgres via Supabase)]
        D[Python Bridge]
        E[Rust Engine (gRPC)]

        C -->|Supabase Realtime| D
        D -->|gRPC| E
        E -->|SQLx| C
    end

    A <-->|Supabase| C
```

### Getting started (dev)

This repo has detailed setup docs per component:
- **Server**: `server/README.txt`
- **Desktop app**: `app/README.txt`

#### Server (recommended path)

The bridge relies on **Supabase Realtime**, so the recommended dev flow is the Supabase CLI + local engine/bridge:

```bash
cd server
tmuxinator start
```

#### App

```bash
cd app
cp .env-example .env   # set VITE_SUPABASE_URL / VITE_SUPABASE_KEY
pnpm install
pnpm tauri dev
```

### Contributing

See `CONTRIBUTING.md`.

### Security

For security issues, please email `security@stoamedical.com` instead of using the issue tracker.

### License

Portico is licensed under the **Business Source License 1.1 (BSL 1.1)**.

- **Free** for internal use processing **≤ 2 production interfaces** or generating **≤ $1M** in trailing-12-month gross charges
- **Change date**: **2030-07-01** (converts to **Apache 2.0**)

See `LICENSE.txt` for full terms.

# Portico -- an analytic interface engine and database system

Portico is an agentic interface engine for scalable and efficient data interchange. It has two modes:
1. `server` which starts a server which can run with concurrency and multi-threading
2. `app` which can connect to a server or run locally with an embedded database

Using first-princple design, Portico strives to be both the most performant and user-friendly option on the market.

## Instructions to run

See the corresponding `README`s:
- [server/README.txt](./server/README.txt)
- [app/README.txt](./app/README.txt)

## Quick Start

Spin up the **entire Portico stack** locally with Docker Compose:

```bash
# clone & enter repository
git clone https://github.com/<your-org>/Portico.git
cd Portico

# build & run all services (Supabase, Bridge, Engine)
docker compose up --build
```

Services started by the Compose file:

- **Supabase** – Postgres + Realtime (ports 54321 / 54322)
- **Bridge** – Python service that converts Supabase events into gRPC calls
- **Engine** – Rust gRPC service that executes workflow steps

Once all containers are healthy you can:

1. Open Supabase Studio at `http://localhost:54323` to inspect the database
2. Start the desktop/web client:
   ```bash
   cd app
   pnpm install
   pnpm tauri dev   # or: pnpm dev for a browser-only preview
   ```

For running individual services, advanced configuration, and development workflows, check the directory-level READMEs linked above.

## Features

### Agent Ownership

Portico supports a configurable agent ownership feature that controls data visibility:

- **Default behavior**: By default, all agents are visible to all users (agent ownership filtering is disabled)
- **Scoped view**: When enabled, users can only view and interact with their own agents

#### Toggling Agent Ownership

There are two ways to toggle the agent ownership feature:

1. **Via Admin Settings UI**:
   - Navigate to the admin settings interface
   - Use the "Enforce Agent Ownership" toggle switch

2. **Programmatically**:
   ```typescript
   import { updateConfig } from "$lib/stores/configStore";

   // Enable agent ownership (restrict users to their own agents)
   updateConfig({ enforceAgentOwnership: true });

   // Disable agent ownership (allow viewing all agents)
   updateConfig({ enforceAgentOwnership: false });
   ```

The setting persists across sessions in the browser's local storage. This feature is particularly useful for:
- Multi-user environments where data isolation is required
- Administrative debugging when you need to view all agents
- Development and testing scenarios

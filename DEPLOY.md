# Deploying Portico

Portico has two independently deployed services and three managed dependencies:

| Component | Deployed to | Source |
|-----------|-------------|--------|
| **Web app** (Next.js) | Vercel | `/` (project root) |
| **Engine + Sidecar** (Rust + Python) | Docker host (Fly.io, Railway, AWS ECS) | `server/` |
| **Database** | Supabase (managed Postgres) | `server/database/schema.sql` |
| **Auth** | Supabase Auth | configured in Supabase dashboard |
| **Clinical data** | Medplum (managed FHIR server) | external |

---

## Local Development

### Prerequisites

- [ ] Node.js 20+ installed
- [ ] pnpm installed
- [ ] Docker & Docker Compose installed
- [ ] [Atlas CLI](https://atlasgo.io/getting-started#installation) installed (`brew install ariga/tap/atlas` on macOS)

### 1. Supabase project

Even for local development, Supabase Auth requires a real project for login to work.

- [ ] Create a project at [supabase.com/dashboard](https://supabase.com/dashboard)
- [ ] Note your **Project URL**, **anon key**, and **service_role key** from **Settings** → **API**

### 2. Configure environment

- [ ] Copy the example env file:
  ```bash
  cp .env.example .env.local
  ```
- [ ] Fill in Supabase keys in `.env.local`:
  ```
  NEXT_PUBLIC_SUPABASE_URL=https://<ref>.supabase.co
  NEXT_PUBLIC_SUPABASE_ANON_KEY=eyJ...
  SUPABASE_SERVICE_ROLE_KEY=eyJ...
  ```
- [ ] Set `DATABASE_URL` to the local Postgres (started by docker-compose):
  ```
  DATABASE_URL=postgresql://postgres:postgres@localhost:54322/postgres
  ```

### 3. Start the server stack

- [ ] Start Postgres, Atlas migrator, Rust engine, and Python sidecar:
  ```bash
  cd server
  docker compose up --build
  ```
- [ ] Verify all services are healthy:
  - Postgres: port 54322
  - Engine health: `curl http://localhost:8080/health`
  - Sidecar health: `curl http://localhost:8001/health`

### 4. Start the web app

- [ ] Install dependencies and start:
  ```bash
  pnpm install
  pnpm dev
  ```
- [ ] Open [http://localhost:3000](http://localhost:3000)
- [ ] Sign up with email/password — confirm via email if Supabase email confirmation is enabled
- [ ] Verify the dashboard loads (zero agents, zero signals is expected)

---

## Production Deployment

### 1. Supabase setup

- [ ] Create a Supabase project (or use the one from local development)
- [ ] Go to **Authentication** → **Providers** and enable Email/Password
- [ ] Optionally enable OAuth providers (Google, GitHub, etc.)
- [ ] Set **Authentication** → **URL Configuration** → **Site URL** to your Vercel deployment URL (e.g., `https://portico.vercel.app`)
- [ ] Note connection details from **Settings** → **API**:

  | Value | Used as |
  |-------|---------|
  | Project URL | `NEXT_PUBLIC_SUPABASE_URL` |
  | `anon` public key | `NEXT_PUBLIC_SUPABASE_ANON_KEY` |
  | `service_role` secret key | `SUPABASE_SERVICE_ROLE_KEY` |

- [ ] Note connection strings from **Settings** → **Database** → **Connection string**:

  | Value | Used as |
  |-------|---------|
  | Transaction pooler (port 6543) | `DATABASE_URL` for the **web app** |
  | Direct connection (port 5432) | `DATABASE_URL` for the **Rust engine** |

  The web app must use the pooler because Vercel Functions are serverless. The engine
  should use the direct connection since it's a long-running process.

### 2. Apply the database schema

- [ ] Export the pooler connection string:
  ```bash
  export DATABASE_URL="postgresql://postgres.[ref]:[password]@aws-0-[region].pooler.supabase.com:6543/postgres"
  ```
- [ ] Apply the schema:
  ```bash
  pnpm db:apply
  ```
- [ ] Verify tables were created (check Supabase dashboard → **Table Editor**)

### 3. Deploy the web app (Vercel)

- [ ] Import the repository at [vercel.com/new](https://vercel.com/new)
- [ ] Framework preset: **Next.js**, root directory: **`.`** (default)
- [ ] Add environment variables in **Settings** → **Environment Variables**:
  ```
  NEXT_PUBLIC_SUPABASE_URL=https://<ref>.supabase.co
  NEXT_PUBLIC_SUPABASE_ANON_KEY=eyJ...
  SUPABASE_SERVICE_ROLE_KEY=eyJ...
  DATABASE_URL=postgresql://postgres.[ref]:[password]@aws-0-[region].pooler.supabase.com:6543/postgres
  MEDPLUM_BASE_URL=https://api.medplum.com
  MEDPLUM_CLIENT_ID=...
  MEDPLUM_CLIENT_SECRET=...
  ENGINE_GRPC_URL=engine.yourdomain.com:50051
  CRON_SECRET=<generate with: openssl rand -hex 32>
  ```
- [ ] Push to `main` — Vercel builds and deploys automatically
- [ ] Verify: visit the deployment URL, sign up, confirm the dashboard loads

### 4. Deploy the engine + sidecar (Docker)

- [ ] Build the Docker images from the repository root:
  ```bash
  docker build -t portico-engine -f server/engine/Dockerfile .
  docker build -t portico-sidecar -f server/python-sidecar/Dockerfile server/python-sidecar
  ```
  Note: the engine Dockerfile uses the repo root as build context (needs `server/proto/`
  and `server/database/`). The sidecar uses `server/python-sidecar/`.

- [ ] Configure engine environment variables:

  | Variable | Required | Description |
  |----------|----------|-------------|
  | `DATABASE_URL` | Yes | Supabase Postgres **direct** connection (port 5432) |
  | `GRPC_PORT` | No | gRPC listen port (default: 50051) |
  | `HEALTH_PORT` | No | Health check HTTP port (default: 8080) |
  | `PYTHON_SIDECAR_URL` | Yes | Sidecar endpoint (e.g., `http://sidecar:8001`) |
  | `MEDPLUM_BASE_URL` | Yes | Medplum FHIR server URL |
  | `MEDPLUM_CLIENT_ID` | Yes | Medplum OAuth client ID |
  | `MEDPLUM_CLIENT_SECRET` | Yes | Medplum OAuth client secret |
  | `AI_GATEWAY_URL` | No | AI Gateway endpoint for LLM steps |
  | `AI_GATEWAY_API_KEY` | No | AI Gateway API key |
  | `REDIS_URL` | No | Upstash Redis for async signal queue |

- [ ] Configure sidecar environment variables:

  | Variable | Required | Description |
  |----------|----------|-------------|
  | `MAX_EXECUTION_TIME` | No | Per-step timeout in seconds (default: 60) |

- [ ] Deploy containers to your Docker host (Fly.io, Railway, AWS ECS, etc.)
- [ ] Ensure both containers are on the same network
- [ ] Verify health checks:
  ```bash
  curl http://<engine-host>:8080/health
  curl http://<sidecar-host>:8001/health
  ```
- [ ] Update `ENGINE_GRPC_URL` in Vercel env vars to point to the deployed engine

### 5. End-to-end verification

- [ ] Sign up / sign in on the deployed web app
- [ ] Dashboard loads with correct counts
- [ ] Create an agent via the UI
- [ ] Send a test signal via the API:
  ```bash
  curl -X POST https://portico.vercel.app/api/signals \
    -H "Content-Type: application/json" \
    -d '{"signalType":"run","source":"test","agentId":"<agent-id>"}'
  ```
- [ ] Verify the signal appears in the dashboard as pending

---

## Schema Changes

The database schema lives in `server/database/schema.sql`, managed declaratively by
Atlas.

- [ ] Edit `server/database/schema.sql`
- [ ] Preview the diff:
  ```bash
  pnpm db:diff
  ```
- [ ] Apply to the database:
  ```bash
  pnpm db:apply
  ```
- [ ] Update TypeScript types in `src/lib/db/types.ts` if you changed columns that the
  web app queries

In the local Docker Compose environment, the schema is applied automatically on startup
by the `schema_migrator` service.

---

## Environment Variable Reference

### Web app (Vercel / local)

| Variable | Required | Description |
|----------|----------|-------------|
| `NEXT_PUBLIC_SUPABASE_URL` | Yes | Supabase project URL |
| `NEXT_PUBLIC_SUPABASE_ANON_KEY` | Yes | Supabase anonymous key |
| `SUPABASE_SERVICE_ROLE_KEY` | Yes | Supabase service role key (server-side only) |
| `DATABASE_URL` | Yes | Postgres connection (pooler for Vercel, direct for local) |
| `MEDPLUM_BASE_URL` | No | Medplum FHIR API URL (default: `https://api.medplum.com`) |
| `MEDPLUM_CLIENT_ID` | No | Medplum OAuth client ID |
| `MEDPLUM_CLIENT_SECRET` | No | Medplum OAuth client secret |
| `ENGINE_GRPC_URL` | No | Engine gRPC endpoint (default: `localhost:50051`) |
| `CRON_SECRET` | Yes | Secret for authenticating Vercel Cron requests |

### Engine (Docker)

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | Postgres **direct** connection (port 5432) |
| `GRPC_PORT` | No | gRPC listen port (default: 50051) |
| `HEALTH_PORT` | No | Health check port (default: 8080) |
| `PYTHON_SIDECAR_URL` | Yes | Python sidecar endpoint |
| `MEDPLUM_BASE_URL` | Yes | Medplum FHIR API URL |
| `MEDPLUM_CLIENT_ID` | Yes | Medplum OAuth client ID |
| `MEDPLUM_CLIENT_SECRET` | Yes | Medplum OAuth client secret |
| `AI_GATEWAY_URL` | No | AI Gateway for LLM steps |
| `AI_GATEWAY_API_KEY` | No | AI Gateway API key |
| `REDIS_URL` | No | Upstash Redis for async signal queue |

### Python sidecar (Docker)

| Variable | Required | Description |
|----------|----------|-------------|
| `MAX_EXECUTION_TIME` | No | Per-step timeout in seconds (default: 60) |

<img
  src="assets/Logo-Inline-Portico.svg"
  alt="Portico Logo"
  style="height: 56px; width: auto; max-width: 100%;"
/>

Portico is a **cloud-native agentic integration engine** for healthcare: an event-driven backend (Signals → Agents → Steps) with a web dashboard for configuration, built for healthcare-style integration workflows.

### Architecture

```mermaid
graph TB
    subgraph "Web App · Next.js on Vercel"
        UI["Dashboard UI<br/>(App Router)"]
        API["API Routes<br/>(webhooks, chat, signals)"]
        LIB["lib/<br/>(agents, db, medplum, analytics)"]
        MW["Supabase Auth Middleware"]
    end

    subgraph "Rust Engine · Docker container"
        ENG["Signal Runner<br/>(tokio)"]
        PY["Python Sidecar<br/>(sandboxed step execution)"]
        ENG <-->|HTTP| PY
    end

    subgraph "Data Layer"
        SUPA[("Supabase Postgres<br/>(operational data)")]
        MED[("Medplum<br/>(FHIR clinical data)")]
    end

    UI --> LIB
    API -->|insert pending signals| SUPA
    ENG -->|lease signals, write results| SUPA
    ENG -->|REST API| MED
    LIB -->|postgres.js| SUPA
    LIB -->|REST API| MED
    ENG -->|HTTP| AIGW["AI Gateway"]
    API -->|AI SDK| AIGW
```

### Tech Stack

| Layer | Technology |
|-------|-----------|
| **Web framework** | Next.js 15 (App Router, React 19) |
| **Auth** | Supabase Auth |
| **Database** | Supabase Postgres (Atlas + postgres.js) |
| **Clinical data** | Medplum (FHIR R4) |
| **AI** | Vercel AI SDK + AI Gateway |
| **Styling** | Tailwind CSS 4, Geist fonts, shadcn/ui |
| **Workflow engine** | Rust (tokio, SQLx) |
| **Python execution** | Python sidecar (FastAPI, sandboxed) |
| **Deployment** | Vercel (web app), Docker (engine + sidecar) |

### Repository Structure

```
├── src/
│   ├── app/                    # Next.js App Router (pages, layouts, API routes)
│   │   ├── (auth)/             #   Login / register pages (Supabase Auth)
│   │   ├── (dashboard)/        #   Dashboard, agents, workflows, mappings, integrations, analytics
│   │   └── api/                #   Webhook endpoints (HL7, FHIR, Medplum), chat, signals, cron
│   ├── lib/                    # Shared TypeScript modules (imported by app/)
│   │   ├── db/                 #   Database client + types (Supabase Postgres)
│   │   ├── supabase/           #   Supabase Auth client utilities (server, browser, middleware)
│   │   ├── agents/             #   Agent types, actions, mapping agent
│   │   ├── medplum/            #   Medplum FHIR client wrapper
│   │   └── analytics/          #   Analytics query functions
│   ├── components/             # React components (UI primitives, sign-out button)
│   └── middleware.ts           # Supabase auth middleware
├── server/                     # Backend services (containerized)
│   ├── engine/                 #   Rust signal runner (leases signals from Postgres, runs steps)
│   ├── python-sidecar/         #   Sandboxed Python step execution
│   ├── database/               #   Atlas SQL schema (source of truth)
│   ├── scripts/e2e.sh          #   End-to-end check against the compose stack
│   └── docker-compose.yml      #   Local dev environment
├── design/                     # Design documents
│   └── 2-cloud-native.md       #   Full architecture design doc
├── next.config.ts              # Next.js config
└── package.json                # Node.js dependencies and scripts
```

### Getting Started

#### Prerequisites

- Node.js 20+
- Docker & Docker Compose (for the Rust engine)
- [Atlas CLI](https://atlasgo.io/getting-started#installation) (for schema management)

#### Web App (Next.js)

```bash
pnpm install
cp .env.example .env.local    # Set Supabase keys, DATABASE_URL, etc.
pnpm run dev                    # http://localhost:3000
```

#### Server (Rust Engine + Python Sidecar)

```bash
cd server
docker compose up --build      # Postgres :54322, Engine health :8080, Sidecar :8001
./scripts/e2e.sh               # seeds an agent, sends signals, asserts on the results
```

See [server/README.md](server/README.md) for detailed setup.

#### Available Scripts

| Script | Description |
|--------|-------------|
| `pnpm dev` | Start Next.js dev server |
| `pnpm build` | Production build |
| `pnpm start` | Start production server |
| `pnpm lint` | ESLint |
| `pnpm db:apply` | Apply Atlas schema to database |
| `pnpm db:diff` | Show pending schema changes |

### Architecture Deep Dive

See [`design/2-cloud-native.md`](design/2-cloud-native.md) for the full design doc covering:
- Rust engine responsibilities (orchestration, parsing, FHIR resource construction)
- Python sidecar architecture (crash isolation, independent scaling)
- Communication patterns (gRPC sync vs Redis Streams async). Neither is implemented yet: the web app inserts `pending` signals and the engine leases them from Postgres.
- Medplum integration strategy
- AI Gateway usage for LLM steps

### Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md).

### Security

For security issues, please email `security@stoamedical.com` instead of using the issue tracker.

### License

Portico is licensed under the **Apache 2.0 License**.

See [`LICENSE.txt`](LICENSE.txt) for full terms.

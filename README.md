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
        ENG["Workflow Orchestrator<br/>(gRPC + tokio)"]
        PY["Python Sidecar<br/>(sandboxed step execution)"]
        ENG <-->|gRPC| PY
    end

    subgraph "Data Layer"
        SUPA[("Supabase Postgres<br/>(operational data)")]
        MED[("Medplum<br/>(FHIR clinical data)")]
        REDIS[("Upstash Redis<br/>(async signals)")]
    end

    UI --> LIB
    API -->|gRPC sync| ENG
    API -->|Redis Streams async| REDIS
    REDIS --> ENG
    ENG -->|SQLx| SUPA
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
| **Cache / async signals** | Upstash Redis |
| **AI** | Vercel AI SDK + AI Gateway |
| **Styling** | Tailwind CSS 4, Geist fonts, shadcn/ui |
| **Workflow engine** | Rust (tokio, gRPC) |
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
│   │   ├── engine/             #   gRPC client for Rust engine
│   │   ├── medplum/            #   Medplum FHIR client wrapper
│   │   └── analytics/          #   Analytics query functions
│   ├── components/             # React components (UI primitives, sign-out button)
│   └── middleware.ts           # Supabase auth middleware
├── server/                     # Backend services (containerized)
│   ├── engine/                 #   Rust gRPC workflow orchestrator
│   ├── python-sidecar/         #   Sandboxed Python step execution
│   ├── database/               #   Atlas SQL schema (source of truth) + Rust DB crate
│   ├── proto/                  #   Protobuf definitions
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
docker compose up --build      # Postgres :54322, Engine :50051, Sidecar :8001
```

Or with tmuxinator for development:

```bash
cd server
tmuxinator start
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
- Communication patterns (gRPC sync vs Redis Streams async)
- Medplum integration strategy
- AI Gateway usage for LLM steps

### Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md).

### Security

For security issues, please email `security@stoamedical.com` instead of using the issue tracker.

### License

Portico is licensed under the **Apache 2.0 License**.

See [`LICENSE.txt`](LICENSE.txt) for full terms.

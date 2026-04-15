*Portico v2* – Design Doc (Dev)

Last significant update: Mar 28, 2026

Overview:

* Healthcare data isn't organized well, and integration engines are still stuck in the 2000s. Portico v2 is a **cloud-native agentic integration engine** that combines a Rust workflow engine with FHIR-native storage (Medplum), AI-powered data mapping, and a modern web UI.
  * v1 proved the core model: Signals, Agents, Steps, RuntimeSessions. v2 takes that model cloud-native and adds healthcare-specific capabilities (HL7v2/FHIR parsing, data mapping, clinical data storage).
* There are 3 main services: the **Rust engine**, the **web app**, and **Medplum**
  * **Rust engine** – containerized workflow orchestrator (evolved from v1)
    * Per-agent MPSC queues (tokio), zero-copy HL7v2/X12 parsing, FHIR resource construction
    * Delegates I/O-bound work: LLM steps via AI Gateway HTTP, FHIR ops via Medplum REST, Python steps via sidecar
    * PyO3 is dropped — Python runs in an isolated sidecar container (crash isolation, any Python version, no GIL interaction)
    * Deployed as a Docker container on Fly.io / Railway / AWS ECS
  * **Web app** – Next.js on Vercel (replaces Tauri desktop app)
    * Agent/workflow configuration, analytics dashboard, AI-assisted data mapping
    * API routes for signal ingestion (webhooks for HL7, FHIR, external systems)
    * Auth via Supabase Auth, data in Supabase Postgres
  * **Medplum** – FHIR-native clinical data repository
    * All clinical data (Patient, Observation, Encounter, etc.) stored as FHIR R4 resources
    * Subscriptions for event-driven workflows
    * SMART on FHIR auth for clinical data access
    * Medplum Cloud or self-hosted

Goals (min 1, max 3):

1. Cloud-native deployment — no local servers, no desktop app installs, accessible from anywhere
2. FHIR-native clinical data — stop reinventing healthcare data models, use Medplum as the clinical data layer
3. AI-powered data mapping — use LLMs to help users build and maintain data mappings between systems

Non-Goals (min 1, max 3):

1. No replacing Medplum's FHIR capabilities — Portico orchestrates data *into* and *out of* Medplum, it doesn't try to be a FHIR server
2. No horizontal scaling of the Rust engine yet — single instance is fine, horizontal comes later
3. No EHR vendor integrations — focus on the data layer, not on connecting to Epic/Cerner/etc. directly

Milestones \+ Timelines (min 1):

1. Phase 1: Rust engine containerized + Next.js scaffold with auth and agent CRUD
2. Phase 2: Medplum integration (FHIR client in Rust, webhook ingestion in Next.js)
3. Phase 3: AI mapping agent + analytics dashboard
4. Phase 4: HL7v2/X12 parsers in Rust, production hardening

---

Key Technical Decisions (min 1):

* **Rust as orchestrator, not runtime-for-everything:** v1 used Rust for everything including Python execution (PyO3). v2 narrows Rust's role to what it's genuinely best at: orchestration, parsing, and data construction. Everything else is delegated.
  * **What Rust owns (hot-path, CPU-bound, correctness-critical):**
    * Channel system + ordering guarantees — tokio MPSC channels give compile-time send/receive ownership, type-safe backpressure (bounded channels force handling "queue full"), true parallelism across agents with no GIL
    * HL7v2/X12 parsing — zero-copy parsing of binary protocols, microsecond latency, no allocations. A hospital ADT feed pushes thousands of messages/sec; Rust parses in microseconds vs. Python in milliseconds
    * FHIR resource construction — struct-heavy work tightly coupled to parser output
    * Schema validation — CPU-bound JSON traversal against FHIR profiles
    * Signal routing + backpressure — deterministic, benefits from Rust's type system (exhaustive match on signal types)
    * Long-running process reliability — no GC pauses, no memory leaks from reference cycles, predictable RSS over weeks of uptime
  * **What Rust delegates (I/O-bound, no performance benefit from Rust):**
    * LLM steps → HTTP call to AI Gateway (network-bound, LLM takes seconds to respond)
    * FHIR CRUD → HTTP call to Medplum REST API (network-bound)
    * Python steps → gRPC call to Python sidecar (see below)
  * **When Node.js (Vercel):** interaction layer — UI, auth, webhooks, AI chat/mapping features, analytics queries, config CRUD

* **Python sidecar instead of PyO3:** v1 embedded Python via PyO3. This pinned the Python version at compile time, coupled Rust builds to Python headers, inherited GIL limitations, and meant a Python crash could take down the engine. v2 runs Python as a separate container (FastAPI or gRPC server) alongside the Rust engine:
  * Rust engine sends step code + input JSON over gRPC (unix socket, ~0.1ms latency)
  * Python sidecar executes in an isolated namespace, returns output JSON
  * **Crash isolation:** Python segfault does not kill the engine — Rust retries or marks the step as failed
  * **Any Python version:** sidecar pins its own Python independently of Rust's build
  * **Normal pip:** `pip install` works, no PyO3 compatibility constraints
  * **Independent scaling:** can run multiple sidecar replicas if Python steps become a bottleneck
  * Both containers share a private network (Docker Compose locally, Fly.io machines in production)

* **Next.js on Vercel instead of Tauri desktop app:** The Tauri app was wrapping a web view to do CRUD operations. There's no filesystem, native menu, or offline requirement. A web app gives: preview deployments for stakeholder review, Server Components for keeping sensitive data server-side, streaming AI features via AI SDK, and no build targets to maintain per OS. If a customer needs on-prem, the Next.js app ships as a Docker container alongside the Rust engine.

* **Medplum as the FHIR data layer:** Medplum provides FHIR R4 storage, subscriptions, SMART on FHIR auth, and US Core compliance out of the box. Portico does not try to store clinical data in its own Postgres — that would mean rebuilding FHIR validation, resource versioning, search parameters, and compliance certifications. Instead:
  * **Medplum owns:** all clinical FHIR resources (Patient, Observation, Encounter, DiagnosticReport, etc.)
  * **Supabase Postgres owns:** Portico's operational data (agents, steps, signals, runtime\_sessions, data\_mappings, audit\_log)
  * The Rust engine reads/writes FHIR via Medplum's REST API and reads/writes operational state via SQLx to Supabase Postgres

* **AI Gateway for all LLM calls:** Instead of hardcoding a single LLM provider, all LLM steps route through Vercel AI Gateway. This gives: automatic failover across providers, cost tracking per agent/step, OIDC auth (no API keys to manage), and the ability to swap models by changing a string. The Rust engine calls AI Gateway over HTTP for LLM steps; the Next.js app uses the AI SDK for interactive AI features (chat, mapping assistant).

* **Communication between Vercel and Rust engine:**
  * **Sync signals** (need a response): gRPC — same proto contract as v1, same `ProcessSignal` / `CreateWorkflow` / `DeleteWorkflow` RPCs
  * **Async signals** (fire-and-forget, high volume): Redis Streams via Upstash — durable, handles backpressure, the engine consumes at its own pace
  * The proto contract (`bridge_message.proto`) is preserved from v1 with additions for new step types

* **Supabase for auth and operational database:** Consolidates auth and Postgres under a single vendor. Supabase Auth provides email/password and OAuth login, session management via cookies, and row-level security (RLS) integration with the database. Supabase Postgres provides connection pooling via Supavisor (important for Vercel Functions) and a direct connection for the Rust engine. Schema managed declaratively by Atlas SQL (`server/database/schema.sql`); queries use raw postgres.js tagged templates. SMART on FHIR auth for clinical data access goes through Medplum directly.

---

Key Data Models:

* **Signal:** (preserved from v1) Something that happens and can trigger actions.
  * Three types: `run` (trigger agent execution with data), `sync` (re-sync engine state), `fyi` (log data)
  * New in v2: `source` field tracks where the signal came from (webhook, medplum-subscription, ui, api)
  * New in v2: signals can carry healthcare message metadata (message type, sending facility, etc.)

* **Agent:** (preserved from v1) An automation unit that does things with Steps.
  * New in v2: agents can specify a preferred LLM model (routed through AI Gateway)
  * New in v2: agents can be associated with data mappings

* **Step:** (evolved from v1) A unit of action. JSON in, Result\<JSON, Error\> out.
  * v1 types: `Python | Prompt | WebScrape`
  * v2 types: `Python | LLM | Transform | Validate | FHIR`
  * Steps are categorized by where they execute:
    * **Native (Rust in-process, CPU-bound):**
      * `Transform` — data format conversion (HL7v2->FHIR, CSV->FHIR, X12->FHIR) using zero-copy Rust parsers
      * `Validate` — FHIR profile validation, JSON Schema validation
    * **Delegated (I/O-bound, Rust awaits result):**
      * `LLM` — HTTP call to AI Gateway with configurable model, system prompt, and optional output schema
      * `FHIR` — HTTP call to Medplum REST API (read/write/search FHIR resources)
      * `Python` — gRPC call to Python sidecar for custom business logic (replaces PyO3)

* **RuntimeSession:** (preserved from v1) Execution record for an agent run.
  * Same as v1: tracks status, step results, step execution times, total execution time
  * New in v2: linked to data quality scores

* **DataMapping:** (new in v2) Configuration for transforming data between formats.
  * Source schema definition, target schema definition (typically a FHIR profile)
  * Field-level mapping rules (can be AI-generated or hand-crafted)
  * Used by Transform steps in the Rust engine

* **Integration:** (new in v2) Configured inbound or outbound connection to an external system.
  * Connection type: `hl7v2-mllp` (TCP listener in engine), `hl7v2-http` (Vercel webhook), `fhir-subscription` (Medplum callback), `rest-webhook`, `file-drop`
  * Associated agent (which agent handles signals from this integration)
  * Health status and last-seen timestamp

* **AuditLog:** (new in v2) Append-only compliance trail.
  * Who did what, when, to which resource
  * Required for healthcare compliance (HIPAA audit requirements)

---

Key Interactions:

* **Signal** (same as v1, new sources):
  * Created by: external webhook (HL7/FHIR/REST), Medplum subscription, UI trigger, API call
  * Creation triggers agent execution via the Rust engine

* **Agent** (same as v1):
  * Contains Steps and a described goal
  * Starts RuntimeSessions
  * New: can be associated with DataMappings and Integrations

* **Step** (expanded from v1):
  * Owned by Agents (Steps are not shared between Agents)
  * Five types: Python, LLM, Transform, Validate, FHIR
  * All assume JSON input and output

* **RuntimeSession** (same as v1):
  * Started by an Agent, represents an individual run
  * Saves runtime duration and success of different steps

* **Medplum** (new):
  * Medplum Subscriptions fire webhooks to Vercel -> create Signals -> Rust engine
  * Rust engine writes FHIR resources to Medplum via REST API (transaction bundles)
  * AI mapping agent reads FHIR profiles from Medplum to validate mappings

---

Architecture:

```
                    External Systems
                    (EHRs, Labs, etc.)
                          │
                    HL7v2 / FHIR / CSV / X12
                          │
                          ▼
┌─────────────────────────────────────────────────────┐
│               Next.js on Vercel                     │
│                                                     │
│  Web UI              API Routes         AI Features │
│  ├── Agent config    ├── POST /signals  ├── Chat    │
│  ├── Workflow builder├── Webhooks       │   (AI SDK │
│  ├── Mapping editor  │   (HL7, FHIR)   │   + AI    │
│  ├── Analytics       ├── Medplum        │   Gateway)│
│  └── Integrations    │   callbacks      └── Mapping │
│                      └── CRUD APIs          agent   │
│                           │                         │
│  Auth: Supabase Auth DB: Supabase Postgres (oper.)  │
└───────────────────────────┬─────────────────────────┘
                            │
                   gRPC (sync) / Redis Streams (async)
                            │
┌───────────────────────────▼─────────────────────────┐
│            Rust Engine (Orchestrator Container)      │
│            Fly.io / Railway / AWS ECS               │
│                                                     │
│  WorkflowManager (per-agent tokio MPSC queues)      │
│  ├── Step dispatch: decides WHERE each step runs    │
│  │                                                  │
│  │  Native steps (Rust in-process, CPU-bound):      │
│  │  ├── TransformStep (HL7v2->FHIR, zero-copy)     │
│  │  ├── ValidateStep  (FHIR profile, JSON Schema)   │
│  │  └── RouteStep     (deterministic data routing)  │
│  │                                                  │
│  │  Delegated steps (I/O-bound, Rust awaits):       │
│  │  ├── LLMStep    ──► HTTP to AI Gateway           │
│  │  ├── FHIRStep   ──► HTTP to Medplum              │
│  │  └── PythonStep ──► gRPC to Python sidecar       │
│  │                                                  │
│  │  HL7v2 Parser (zero-copy) · X12 Parser           │
│  │  FHIR Resource Builder · MedplumClient           │
│  │                                                  │
│  │  DB: Supabase Postgres via SQLx (operational)    │
│  └──────────────────────────────────────────────────┘
│         │ gRPC (unix socket, ~0.1ms)                │
│         ▼                                           │
│  ┌──────────────────────────────────────────┐       │
│  │  Python Sidecar (Container)              │       │
│  │  FastAPI or gRPC server                  │       │
│  │  ├── Receives step code + input JSON     │       │
│  │  ├── Executes in isolated namespace      │       │
│  │  ├── Returns output JSON                 │       │
│  │  ├── Crash-isolated from Rust engine     │       │
│  │  └── Any Python version, normal pip      │       │
│  └──────────────────────────────────────────┘       │
└──────────┬──────────────────────────────────────────┘
           │ REST (FHIR R4)
           ▼
┌──────────────────────┐         ┌────────────────────┐
│  Medplum             │         │  Supporting Infra   │
│  (FHIR Server)       │         │                    │
│                      │         │  Supabase Postgres     │
│  Patient, Obs,       │         │  (operational DB)  │
│  Encounter, etc.     │         │                    │
│  Subscriptions       │         │  Upstash Redis     │
│  SMART on FHIR       │         │  (cache + queue)   │
│                      │         │                    │
│  Cloud or self-hosted│         │  Vercel Blob       │
└──────────────────────┘         │  (file uploads)    │
                                 └────────────────────┘
```

Rust responsibility matrix:

| Component | In Rust? | Why |
|---|---|---|
| MPSC queue system | Yes | Compile-time channel safety, true parallelism, type-safe backpressure |
| Signal routing | Yes | Core orchestration, exhaustive match on signal types |
| HL7v2/X12 parsers | Yes | CPU-bound, zero-copy, the real performance win |
| FHIR resource construction | Yes | Struct-heavy, tight coupling with parser output |
| Schema validation | Yes | CPU-bound JSON traversal |
| RuntimeSession bookkeeping | Yes | Runs in the hot loop, benefits from no-GC |
| Python step execution | No — sidecar | Crash isolation, any Python version, no GIL interaction |
| LLM calls | No — HTTP out | Network-bound, just `reqwest::get()` waiting on AI Gateway |
| FHIR CRUD (Medplum) | No — HTTP out | Network-bound, just REST calls |

---

PHI Boundaries \+ Compliance:

Every component in the system is classified by whether it is **permitted to process PHI** (Protected Health Information). This is not a detail to fill in later — it constrains which services can talk to each other and what data can flow where.

**PHI classification by component:**

| Component | Handles PHI? | BAA required? | Notes |
|---|---|---|---|
| **Medplum** | Yes — primary PHI store | Yes (Medplum Cloud provides BAA; self-hosted = your responsibility) | All clinical data lives here. FHIR resources are PHI by definition. |
| **Rust engine** | Yes — transient PHI in memory | Yes (your infrastructure — Fly.io/Railway/ECS must support BAA) | Parses HL7v2/FHIR messages containing PHI. Holds PHI in memory during step execution. Does not persist PHI — writes to Medplum or Supabase Postgres, then drops. |
| **Python sidecar** | Yes — transient PHI in memory | Same as engine (same infrastructure) | Executes user-written steps that may operate on clinical data. Same transient-only rule. |
| **Supabase Postgres** | Conditional — operational data may reference PHI | Yes if signals/runtime\_sessions contain clinical payloads | Signal `payload` and `response_data` fields may contain PHI. Options: (a) store only FHIR resource references (Patient/123), not inline data, or (b) treat Supabase Postgres as PHI-capable and sign Supabase Postgres's BAA. Decision required before production. |
| **Upstash Redis** | Conditional — async queue payloads | Yes if signal payloads flow through Redis Streams | Same decision as Supabase Postgres: reference-only payloads avoid PHI in Redis. If payloads contain clinical data, Redis needs BAA coverage. |
| **Vercel (Next.js)** | Transient — webhook ingestion | Evaluate for production | Webhook endpoints receive raw HL7v2/FHIR payloads containing PHI. Payloads are in transit only (TLS-encrypted, not persisted on Vercel), but Vercel Functions do parse and validate message content before dispatching to the engine. If processing goes beyond simple dispatch (e.g., extracting fields for signal metadata), BAA coverage with Vercel should be evaluated. |
| **Supabase Auth** | No | No | Auth only. No clinical data touches Supabase Auth. |
| **AI Gateway / LLM providers** | **Requires explicit decision** | If yes, provider must have BAA | See below. |

**AI Gateway and PHI — the hard constraint:**

LLM steps (`LLMStep`) can be configured to process clinical data (e.g., "extract diagnosis from this note"). This sends PHI to an external LLM provider. The architecture must enforce one of:

1. **De-identification before LLM:** Transform steps strip PHI (names, MRNs, dates) before the LLM step runs. The LLM only sees de-identified data. This is the default-safe approach.
2. **BAA-covered LLM provider:** Use a provider with a signed BAA (e.g., Azure OpenAI with HIPAA BAA, or AWS Bedrock). Route through AI Gateway with `only: ['azure-openai']` to restrict to compliant providers.
3. **No clinical data in LLM steps:** Restrict LLM steps to operational tasks (mapping generation, schema inference) where the input is metadata, not patient data.

The agent's `policy` field (JSONB) must include a `phi_llm_policy` enum: `deny | deidentify_first | baa_provider_only`. The engine enforces this before dispatching any LLM step.

**Tenant isolation:**

* v2 is single-tenant (one Portico deployment = one organization). Multi-tenant is a later-phase feature.
* When multi-tenant ships: Medplum projects provide per-tenant FHIR isolation; Supabase Postgres row-level security (RLS) provides per-tenant operational isolation; the engine routes signals to tenant-scoped agents.

---

Signal Delivery Semantics:

An integration engine that can duplicate writes to Medplum or silently drop messages is not production-credible. This section defines the idempotency, retry, and ordering guarantees.

**Idempotency model:**

Every Signal gets an `idempotency_key` at creation time. The key is derived from the source:
* Webhook signals: hash of (source\_integration\_id + message\_control\_id + timestamp). HL7v2 MSH-10 (message control ID) is the natural dedupe key.
* Medplum subscription signals: hash of (subscription\_id + resource\_id + version\_id). FHIR resource versioning provides natural idempotency.
* UI/API signals: caller-provided idempotency key, or UUID if not provided.

The `signals` table enforces a **unique constraint on `idempotency_key`**. Duplicate inserts return the existing signal's ID and status — no re-processing.

```
signals
├── ...
├── idempotency_key     TEXT NOT NULL UNIQUE  -- dedupe key
├── ...
```

**Retry policy:**

When a step fails, the engine applies the retry policy defined on the agent:

```
agents.policy.retry = {
  max_attempts: 3,          -- per-step retry limit
  backoff: "exponential",   -- constant | linear | exponential
  base_delay_ms: 1000,      -- initial delay
  max_delay_ms: 30000,      -- cap
  retryable_errors: ["timeout", "transient", "sidecar_crash"]
}
```

* **Retryable failures:** network timeouts, sidecar crashes, Medplum 5xx, AI Gateway 429/503.
* **Fatal failures:** validation errors, Python exceptions (business logic), Medplum 4xx (bad FHIR resource). These mark the step as failed immediately — no retry.
* After `max_attempts`, the signal transitions to `failed` status with `error_message` populated.

**Poison message handling:**

Signals that fail all retries are written to a `dead_letter` table:

```
dead_letter_signals
├── id                  UUID PK
├── signal_id           UUID FK -> signals
├── agent_id            UUID FK -> agents
├── failure_reason      TEXT
├── last_step_index     INT          -- which step failed
├── last_error          JSONB        -- full error context
├── original_payload    JSONB
├── created_at          TIMESTAMPTZ
├── resolved_at         TIMESTAMPTZ  -- NULL until manually resolved
├── resolved_by         TEXT         -- user who resolved
```

The dashboard surfaces dead-letter signals with a "retry" action that re-creates the signal (new idempotency key, references the original).

**Ordering guarantees:**

* **Within an agent:** sequential. The MPSC queue processes one signal at a time per agent. Retries go back to the front of that agent's queue.
* **Across agents:** parallel, no ordering guarantee.
* **Redis Streams (async path):** consumer group with single consumer per stream = ordered delivery. If the engine restarts, it resumes from the last acknowledged message ID (Redis `XACK`).
* **gRPC (sync path):** request-response, inherently ordered per-caller.

**FHIRStep idempotency:**

FHIR transaction bundles sent to Medplum use `PUT` with `If-Match` (version-aware update) or conditional creates (`If-None-Exist`). This prevents duplicate resource creation on retry. The engine tracks the Medplum response's `ETag` / version in the RuntimeSession step results.

---

Python Sidecar Sandboxing:

"Isolated namespace" is not a security model. Users submit arbitrary Python code. Without real sandboxing, this is remote code execution. The sidecar must enforce hard limits.

**Execution sandbox model:**

Each step execution runs in a **fresh subprocess** within the sidecar container, not the main server process. The subprocess is created with:

* **CPU limit:** cgroup or `ulimit` — default 1 CPU, configurable per-agent policy
* **Memory limit:** cgroup — default 256MB, max 1GB, configurable per-agent policy
* **Time limit:** `MAX_EXECUTION_TIME` enforced by the sidecar — default 30s, max 300s. The subprocess is `SIGKILL`ed on timeout.
* **Filesystem:** read-only root filesystem. A small tmpdir (50MB) mounted at `/tmp` for scratch. Cleared after each execution.
* **Network:** **no egress by default**. The sidecar container's network policy blocks all outbound connections except the unix socket back to the engine. Steps that need network access (e.g., calling an external API) must be explicitly allowlisted in the agent's `policy.python_network_allowlist` (list of host:port patterns).
* **Package allowlist:** pip packages are pre-installed at sidecar image build time, not at runtime. Users cannot `pip install` during execution. The sidecar image includes a curated set: `pandas`, `numpy`, `hl7apy`, `fhirclient`, `jsonschema`, `requests` (only functional if network is allowlisted). Custom packages require a sidecar image rebuild.
* **Import restrictions:** `os.system`, `subprocess`, `socket` (direct), `ctypes`, `importlib` are blocked via a custom import hook. `open()` is restricted to `/tmp`.
* **Secret scoping:** environment variables in the sidecar are limited to `INPUT_DATA` (the step's JSON input). No database credentials, API keys, or Medplum tokens are exposed to user code. If a step needs to call an external service, it should be a dedicated step type (LLM, FHIR, etc.), not Python with leaked credentials.

**Defense in depth (container level):**

* Sidecar runs as non-root user (`portico-executor`, UID 1001)
* `no-new-privileges` security option
* Capabilities dropped: all except those needed for cgroup enforcement
* Seccomp profile: default Docker seccomp (blocks `mount`, `reboot`, `kexec_load`, etc.)
* If deployed on ECS/Fargate: task-level IAM role has zero permissions (no AWS API access)

**Monitoring:**

* Each execution logs: CPU time used, peak memory, wall-clock time, exit code
* Steps killed by timeout or OOM are tagged in RuntimeSession results with `killed_reason: "timeout" | "oom"`
* Repeated kills from the same agent trigger an alert in the dashboard

---

Cross-Database Consistency (Medplum \+ Supabase Postgres):

A workflow writes FHIR resources to Medplum and operational state to Supabase Postgres. These are separate databases with no distributed transaction. Partial failure will happen. The design must handle it.

**Outbox pattern for Medplum writes:**

The engine does not write to Medplum and Supabase Postgres independently. Instead:

1. **Step executes** — produces a FHIR Bundle and step result
2. **Engine writes to Supabase Postgres first** — inserts a row in `outbox_events` table within the same transaction as the RuntimeSession update:

```
outbox_events
├── id                  UUID PK
├── signal_id           UUID FK -> signals
├── step_index          INT
├── event_type          TEXT         -- medplum_transaction | medplum_create | medplum_update
├── payload             JSONB        -- the FHIR Bundle to send
├── status              TEXT         -- pending | sent | confirmed | failed
├── medplum_response    JSONB        -- Medplum's response (populated after send)
├── retry_count         INT DEFAULT 0
├── created_at          TIMESTAMPTZ
├── sent_at             TIMESTAMPTZ
```

3. **Outbox publisher** (background task in the engine) polls `outbox_events WHERE status = 'pending'`, sends to Medplum, and marks `confirmed` or `failed`.
4. If Medplum is down, the outbox retries with exponential backoff. The RuntimeSession is already committed to Supabase Postgres — the UI shows "FHIR write pending" rather than an inconsistent state.

**Failure scenarios and recovery:**

| Scenario | What happens | Recovery |
|---|---|---|
| Medplum transaction succeeds, Supabase Postgres update fails | Supabase Postgres transaction rolls back. Outbox event never created. Medplum has the data but Portico doesn't know. | Engine retries the full step. FHIRStep uses conditional creates (`If-None-Exist`) so the Medplum retry is idempotent. |
| Supabase Postgres outbox written, Medplum send fails | Outbox event stays `pending`. RuntimeSession shows step as "pending\_fhir\_write". | Outbox publisher retries. After max retries, marks `failed` and surfaces in dashboard. |
| Engine crashes mid-step | Supabase Postgres transaction was never committed. Signal stays `dispatched`. | On restart, engine re-processes `dispatched` signals from Supabase Postgres. Idempotency keys prevent duplicate processing. |
| Engine crashes after Supabase Postgres commit, before Medplum send | Outbox event exists as `pending`. | Outbox publisher picks it up on restart. |

**Reconciliation job (cron):**

A scheduled job (Vercel cron or engine background task) runs every 15 minutes:
* Queries `outbox_events WHERE status = 'pending' AND created_at < now() - interval '5 minutes'`
* Re-attempts Medplum sends for stuck events
* Queries Medplum for resources that should exist (by idempotency reference) and reconciles with Supabase Postgres state
* Logs discrepancies to `audit_log`

---

HL7v2 Ingestion:

The doc said "webhooks for HL7" but classic HL7v2 arrives over MLLP/TCP (Minimal Lower Layer Protocol), not HTTP. The architecture must account for both transport modes.

**Two ingestion paths:**

1. **MLLP/TCP listener (traditional hospital feeds):**
   * HL7v2 over MLLP is a TCP connection with a specific framing protocol (VT/FS/CR delimiters). This cannot run on Vercel — it needs a persistent TCP socket.
   * The **MLLP listener runs in the engine container** (or a dedicated ingress sidecar). Rust is well-suited for this: `tokio::net::TcpListener` with a custom MLLP codec.
   * On receiving an HL7v2 message, the listener: (a) sends an ACK/NAK back to the sender per HL7v2 protocol, (b) creates a Signal in Supabase Postgres, (c) queues for processing.
   * This is the path for direct hospital interface engine connections (ADT feeds, lab results, orders).

2. **HTTP webhook (modern integrations):**
   * Some systems (integration engines like Mirth/Rhapsody, cloud EHRs, Health Gorilla, etc.) can send HL7v2 wrapped in HTTP POST bodies.
   * This path uses the Vercel API route (`/api/webhooks/hl7`). The message body is the raw HL7v2 string.
   * Vercel validates the HTTP envelope, creates the Signal, dispatches to the engine.

**Architecture update:**

```
Hospital ADT feed (MLLP/TCP)
        │
        ▼
┌───────────────────────┐
│  MLLP Listener        │ ◄── Rust, runs in engine container
│  tokio TcpListener    │     or dedicated ingress sidecar
│  ├── Parse MLLP frame │
│  ├── Send ACK/NAK     │
│  └── Create Signal    │
└───────────┬───────────┘
            │ (internal, same process or gRPC)
            ▼
      Engine WorkflowManager

Cloud integration (HTTP)
        │
        ▼
┌───────────────────────┐
│  /api/webhooks/hl7    │ ◄── Vercel Function
│  ├── Validate body    │
│  └── Create Signal    │────► Redis Streams ────► Engine
└───────────────────────┘
```

The `integrations` table's `connection_type` field distinguishes the two:
* `hl7v2-mllp` — engine-hosted TCP listener. Config includes `listen_port`, `sending_facility_filter`.
* `hl7v2-http` — Vercel webhook. Config includes `endpoint_path`, `auth_header`.

---

Engine Durability (Single-Node):

"No horizontal scaling yet" is fine as a non-goal, but a single-node engine with in-memory queues must still survive deploys, crashes, and restarts gracefully. Without this, "cloud-native" is aspirational.

**Queue reconstruction on startup:**

1. Engine starts and connects to Supabase Postgres.
2. Queries `signals WHERE status IN ('pending', 'dispatched') ORDER BY created_at ASC`.
3. For each signal: looks up the associated agent, creates the agent's MPSC queue if it doesn't exist, and enqueues the signal.
4. Also resumes the Redis Streams consumer group from the last `XACK`ed message ID (persisted in Supabase Postgres as `engine_state.last_redis_stream_id`).
5. Engine is now caught up. New signals flow in via gRPC or Redis Streams.

**Graceful shutdown (deploy/restart):**

1. Engine receives `SIGTERM` (Fly.io/Railway/ECS send this before killing the container).
2. Engine stops accepting new gRPC connections and Redis Stream reads.
3. Engine drains in-flight signals: waits up to 30 seconds for currently-executing steps to complete.
4. For signals still in MPSC queues (not yet started): marks them as `pending` in Supabase Postgres (they were `dispatched` when queued).
5. Persists `engine_state.last_redis_stream_id` to Supabase Postgres.
6. Engine exits cleanly.

If the engine is `SIGKILL`ed (hard crash, OOM):
* In-flight signals stay as `dispatched` in Supabase Postgres. On restart, the reconstruction query picks them up.
* The Redis consumer group's last `XACK` position may be slightly behind — Redis re-delivers unacknowledged messages. Idempotency keys prevent duplicate processing.

**Lease model for in-flight signals:**

To distinguish "dispatched and being worked on" from "dispatched and the engine crashed":

```
signals
├── ...
├── leased_at           TIMESTAMPTZ  -- set when engine dequeues for processing
├── lease_expires_at    TIMESTAMPTZ  -- leased_at + agent's max step duration
├── ...
```

A cron job (or the engine's startup reconstruction) reclaims signals where `status = 'dispatched' AND lease_expires_at < now()` — these are signals from a crashed engine that were never completed.

**Health check:**

The engine exposes a `/health` HTTP endpoint (Tonic health service or a small Hyper sidecar):
* `200 OK` — engine is accepting signals, queues are operational
* `503` — engine is draining (graceful shutdown) or unhealthy
* Fly.io/Railway health checks hit this endpoint to trigger restarts on failure.

---

Key Workflow (v2):

1. External system sends a healthcare message (HL7v2 ADT, FHIR Bundle, CSV file, etc.)
2. Next.js webhook endpoint receives the message, validates format, creates a Signal in Supabase Postgres
3. Signal is dispatched to the Rust engine:
   * Sync path (gRPC): for signals that need an immediate response
   * Async path (Redis Streams): for high-volume fire-and-forget signals
4. Rust engine routes the Signal to the appropriate Agent via WorkflowManager
   1. Agent's MPSC queue receives the Signal
   2. Steps execute sequentially:
      * `TransformStep`: Rust parser converts HL7v2/X12 to FHIR resources (zero-copy)
      * `ValidateStep`: validates FHIR resources against profiles
      * `FHIRStep`: writes validated resources to Medplum via transaction bundle
      * `LLMStep` (optional): AI-assisted data mapping or enrichment via AI Gateway
      * `PythonStep` (optional): custom business logic via Python sidecar
   3. RuntimeSession created with step results and timing data
5. RuntimeSession saved to Supabase Postgres
6. Signal updated with result and RuntimeSession reference
7. Web UI reflects the updated state via Server Components (no Realtime subscription needed — poll or SSE)

Alternative trigger — Medplum Subscription:
1. A FHIR resource is created/updated in Medplum
2. Medplum Subscription fires a webhook to Next.js endpoint
3. Next.js creates a Signal and dispatches to the Rust engine
4. Engine runs agent steps (analytics, alerting, downstream transforms)

---

Directory Structure:

```
portico/
├── app/                          # Next.js App Router (Vercel)
│   ├── (auth)/
│   │   ├── login/page.tsx
│   │   └── register/page.tsx
│   ├── (dashboard)/
│   │   ├── layout.tsx            # Sidebar, auth guard
│   │   ├── page.tsx              # System overview
│   │   ├── agents/
│   │   │   ├── page.tsx          # Agent list
│   │   │   └── [id]/page.tsx     # Agent detail + step editor
│   │   ├── workflows/
│   │   │   ├── page.tsx          # Workflow list
│   │   │   └── [id]/page.tsx     # Workflow detail + run history
│   │   ├── mappings/
│   │   │   ├── page.tsx          # Data mapping list
│   │   │   └── [id]/page.tsx     # Mapping editor (AI-assisted)
│   │   ├── integrations/
│   │   │   └── page.tsx          # Inbound/outbound connections
│   │   └── analytics/
│   │       └── page.tsx          # Runtime metrics + data quality
│   ├── api/
│   │   ├── signals/route.ts      # POST — dispatch signals to engine
│   │   ├── webhooks/
│   │   │   ├── hl7/route.ts      # HL7v2 message ingestion
│   │   │   ├── fhir/route.ts     # FHIR bundle ingestion
│   │   │   └── medplum/route.ts  # Medplum subscription callbacks
│   │   ├── chat/route.ts         # AI mapping assistant
│   │   └── cron/
│   │       └── cleanup/route.ts  # Scheduled maintenance
│   └── proxy.ts                  # Supabase auth middleware
│
├── lib/
│   ├── db/
│   │   ├── types.ts              # TypeScript row types (operational tables)
│   │   ├── client.ts             # postgres.js client (lazy init)
│   │   └── migrations/
│   ├── medplum/
│   │   ├── client.ts             # Medplum SDK client
│   │   └── subscriptions.ts      # Manage Medplum subscriptions
│   ├── engine/
│   │   ├── client.ts             # gRPC client to Rust engine
│   │   └── proto/                # Generated TS protobuf types
│   ├── agents/
│   │   ├── types.ts              # Shared TS types
│   │   ├── actions.ts            # Server Actions for agent CRUD
│   │   └── mapping-agent.ts      # AI mapping agent (AI SDK)
│   └── analytics/
│       └── queries.ts            # Analytics SQL queries
│
├── components/
│   ├── ui/                       # shadcn/ui primitives
│   ├── ai-elements/              # AI Elements (chat, message rendering)
│   ├── agents/                   # Agent CRUD components
│   ├── workflows/                # Workflow builder + step editor
│   ├── mappings/                 # Mapping editor
│   ├── analytics/                # Charts, tables
│   └── editor/                   # CodeMirror Python editor
│
├── server/                       # Rust engine (deploys as container)
│   ├── engine/
│   │   └── src/
│   │       ├── main.rs           # Tonic gRPC server
│   │       ├── lib.rs
│   │       ├── core/
│   │       │   ├── rpc_server.rs
│   │       │   └── workflow_manager.rs
│   │       ├── handlers/
│   │       │   ├── run.rs
│   │       │   ├── sync.rs
│   │       │   ├── fyi.rs
│   │       │   ├── create.rs
│   │       │   └── delete.rs
│   │       ├── services/
│   │       │   ├── fhir_client.rs    # Medplum REST client (reqwest)
│   │       │   ├── hl7_parser.rs     # Zero-copy HL7v2 parser
│   │       │   ├── x12_parser.rs     # X12/EDI parser
│   │       │   ├── mllp_listener.rs  # MLLP/TCP ingress for HL7v2 feeds
│   │       │   ├── fhir_transform.rs # Message format -> FHIR resources
│   │       │   ├── outbox.rs         # Outbox publisher (Medplum write-ahead)
│   │       │   ├── workflow_planner.rs
│   │       │   ├── agent_manager.rs
│   │       │   ├── agent_monitoring.rs
│   │       │   └── agent_cache.rs
│   │       └── steps/
│   │           ├── python.rs         # gRPC client to Python sidecar
│   │           ├── llm.rs            # AI Gateway HTTP call (delegated)
│   │           ├── transform.rs      # HL7v2/X12->FHIR (native, in-process)
│   │           ├── validate.rs       # FHIR profile validation (native, in-process)
│   │           └── fhir.rs           # Medplum CRUD operations (delegated)
│   ├── database/                     # Shared Rust crate (SQLx)
│   │   └── src/
│   │       ├── lib.rs
│   │       └── models/
│   │           ├── agents/
│   │           ├── workflows/
│   │           ├── steps/
│   │           ├── signals/
│   │           ├── runtime_sessions/
│   │           ├── data_mappings/    # New
│   │           ├── integrations/     # New
│   │           └── audit_log/        # New
│   ├── python-sidecar/              # Python step execution (separate container)
│   │   ├── server.py                # FastAPI/gRPC server for step execution
│   │   ├── executor.py              # Isolated namespace execution
│   │   ├── requirements.txt         # Python dependencies (any version)
│   │   └── Dockerfile               # Standalone Python container
│   ├── proto/
│   │   ├── bridge_message.proto     # Engine <-> Web app (evolved from v1)
│   │   └── python_step.proto        # Engine <-> Python sidecar
│   ├── Dockerfile                    # Multi-stage build (Rust engine only)
│   ├── docker-compose.yml           # Engine + Python sidecar (shared network)
│   └── fly.toml                      # Fly.io deployment config
│
├── next.config.ts
├── vercel.ts                         # Vercel project config (crons, rewrites)
└── package.json
```

---

Database Schema (v2):

**Supabase Postgres (operational — managed by Atlas SQL):**

```
agents
├── id                  UUID PK
├── name                TEXT NOT NULL
├── description         TEXT
├── capabilities        JSONB          -- tool access, model preference, max steps
├── policy              JSONB          -- rate limits, execution constraints
├── preferred_model     TEXT           -- AI Gateway model string (e.g. "anthropic/claude-sonnet-4.6")
├── created_at          TIMESTAMPTZ
└── updated_at          TIMESTAMPTZ

steps
├── id                  UUID PK
├── agent_id            UUID FK -> agents
├── name                TEXT NOT NULL
├── step_type           TEXT NOT NULL   -- python | llm | transform | validate | fhir
├── step_order          INT NOT NULL
├── config              JSONB          -- type-specific config (code, prompt, model, fhir_operation, etc.)
├── created_at          TIMESTAMPTZ
└── updated_at          TIMESTAMPTZ

signals
├── id                  UUID PK
├── signal_type         TEXT NOT NULL   -- run | sync | fyi
├── source              TEXT NOT NULL   -- webhook | medplum-subscription | ui | api
├── idempotency_key     TEXT NOT NULL UNIQUE  -- dedupe key (derived from source)
├── agent_id            UUID FK -> agents
├── payload             JSONB          -- input data (may be reference-only for PHI safety)
├── response_data       JSONB          -- result after processing
├── runtime_session_id  UUID FK -> runtime_sessions
├── source_metadata     JSONB          -- message type, sending facility, etc.
├── status              TEXT NOT NULL   -- pending | dispatched | completed | failed
├── error_message       TEXT
├── leased_at           TIMESTAMPTZ    -- set when engine dequeues
├── lease_expires_at    TIMESTAMPTZ    -- leased_at + max step duration
├── created_at          TIMESTAMPTZ
└── updated_at          TIMESTAMPTZ

runtime_sessions
├── id                  UUID PK
├── agent_id            UUID FK -> agents
├── signal_id           UUID FK -> signals
├── status              TEXT NOT NULL   -- waiting | running | completed | failed | cancelled
├── step_results        JSONB[]        -- per-step output
├── step_execution_ms   INT[]          -- per-step timing
├── total_execution_ms  INT
├── data_quality_score  REAL           -- 0.0 to 1.0
├── created_at          TIMESTAMPTZ
└── completed_at        TIMESTAMPTZ

data_mappings
├── id                  UUID PK
├── name                TEXT NOT NULL
├── source_schema       JSONB          -- description of source format
├── target_schema       JSONB          -- description of target (usually FHIR profile)
├── field_mappings      JSONB          -- array of {source_path, target_path, transform}
├── ai_generated        BOOLEAN        -- true if created by AI mapping agent
├── agent_id            UUID FK -> agents (optional)
├── created_at          TIMESTAMPTZ
└── updated_at          TIMESTAMPTZ

integrations
├── id                  UUID PK
├── name                TEXT NOT NULL
├── connection_type     TEXT NOT NULL   -- hl7v2-mllp | hl7v2-http | fhir-subscription | rest-webhook | file-drop
├── direction           TEXT NOT NULL   -- inbound | outbound
├── config              JSONB          -- endpoint URL, auth credentials ref, format options
├── agent_id            UUID FK -> agents
├── status              TEXT NOT NULL   -- active | inactive | error
├── last_seen_at        TIMESTAMPTZ
├── created_at          TIMESTAMPTZ
└── updated_at          TIMESTAMPTZ

dead_letter_signals
├── id                  UUID PK
├── signal_id           UUID FK -> signals
├── agent_id            UUID FK -> agents
├── failure_reason      TEXT
├── last_step_index     INT            -- which step failed
├── last_error          JSONB          -- full error context
├── original_payload    JSONB
├── created_at          TIMESTAMPTZ
├── resolved_at         TIMESTAMPTZ    -- NULL until manually resolved
└── resolved_by         TEXT           -- user who resolved

outbox_events
├── id                  UUID PK
├── signal_id           UUID FK -> signals
├── step_index          INT
├── event_type          TEXT NOT NULL   -- medplum_transaction | medplum_create | medplum_update
├── payload             JSONB          -- FHIR Bundle to send
├── status              TEXT NOT NULL   -- pending | sent | confirmed | failed
├── medplum_response    JSONB
├── retry_count         INT DEFAULT 0
├── created_at          TIMESTAMPTZ
└── sent_at             TIMESTAMPTZ

engine_state
├── id                  INT PK DEFAULT 1  -- singleton row
├── last_redis_stream_id TEXT           -- consumer group position for restart recovery
├── last_heartbeat      TIMESTAMPTZ
└── updated_at          TIMESTAMPTZ

audit_log
├── id                  BIGSERIAL PK
├── actor               TEXT NOT NULL   -- user ID or system identifier
├── action              TEXT NOT NULL   -- created | updated | deleted | executed
├── resource_type       TEXT NOT NULL   -- agent | step | signal | mapping | integration
├── resource_id         UUID
├── details             JSONB
├── created_at          TIMESTAMPTZ
```

**Medplum (clinical — FHIR R4 resources):**

Medplum stores all clinical data as standard FHIR R4 resources. Portico does not define or manage this schema — Medplum handles resource validation, versioning, search indexing, and access control. Common resource types that Portico workflows interact with:

* Patient, Encounter, Observation, DiagnosticReport
* Bundle (transaction bundles for batch writes)
* Subscription (triggers for Portico signals)
* StructureDefinition (FHIR profiles for validation)

---

What Changed from v1:

| Concern | v1 | v2 | Why |
|---|---|---|---|
| Frontend | Tauri + SvelteKit desktop app | Next.js web app on Vercel | No native OS needs; web gives preview deploys, SSR, streaming AI, team access |
| Auth | Supabase Auth | Supabase Auth | Consolidated with database under one vendor; email/password + OAuth, RLS integration, self-hostable |
| Database | Supabase Postgres | Supabase Postgres | Unified with auth provider, connection pooling via Supavisor, self-hostable for on-prem |
| Clinical data | None (planned) | Medplum (FHIR R4) | Don't rebuild FHIR compliance — Medplum owns clinical data |
| Event bridge | Python asyncio + Supabase Realtime | gRPC (sync) + Redis Streams (async) | Eliminates the bridge service; webhooks + queue replace Realtime |
| LLM provider | Direct API key to together.ai | AI Gateway (OIDC, failover, cost tracking) | Provider-agnostic, observable, no keys to manage |
| Cache | In-memory Rust HashMap | Upstash Redis | Survives restarts, shared across instances when scaling |
| Step types | Python, Prompt, WebScrape | Python, LLM, Transform, Validate, FHIR | Healthcare-specific steps; native vs. delegated split |
| Python execution | PyO3 (embedded in Rust) | Python sidecar (separate container) | Crash isolation, any Python version, no GIL, normal pip |
| Engine role | Rust does everything | Rust orchestrates + parses; delegates I/O | Rust for what it's good at (channels, parsing), not for HTTP waits |
| Engine deploy | Docker Compose (local) | Fly.io / Railway / ECS (cloud) | Cloud-native, auto-restart, health checks, zero-downtime deploys |

What is preserved from v1:

* **Data model concepts:** Signal (run/sync/fyi), Agent, Step, RuntimeSession — all carry forward
* **Engine architecture:** per-agent MPSC queues, sequential within agent, parallel across agents
* **Proto contract:** `bridge_message.proto` evolves with new step types, but existing RPCs are unchanged
* **Step I/O contract:** JSON in, Result\<JSON, Error\> out
* **Python step capability:** users can still write Python steps; execution moves from PyO3 (embedded) to sidecar (isolated container)
* **SQLx for database access:** compile-time checked queries

---

Deployment:

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Vercel         │     │   Fly.io/Railway  │     │   Medplum       │
│                  │     │                  │     │   (Cloud or     │
│   Next.js app    │────>│   Rust engine    │────>│    self-hosted) │
│   API routes     │gRPC │   + Python       │REST │                 │
│   AI features    │     │     sidecar      │     │   FHIR R4 API   │
│                  │     │                  │     │   PostgreSQL    │
└────────┬─────────┘     └────────┬─────────┘     └─────────────────┘
         │                        │
    ┌────▼────────────────────────▼──┐
    │       Supabase Postgres            │
    │        (operational DB)        │
    └────────────────────────────────┘
    ┌────────────────┐
    │  Upstash Redis │  (cache, async signal queue)
    └────────────────┘
```

CI/CD:

* **Web app:** push to main -> Vercel auto-deploys. PRs get preview deployments.
* **Rust engine:** push to main -> GitHub Actions builds Docker image -> deploys to Fly.io/Railway
* **Database:** Atlas schema applied as part of the deploy pipeline (`atlas schema apply`)
* **Medplum:** managed separately (Medplum Cloud) or as part of infra (self-hosted via AWS CDK)

---

Environment Variables:

| Service | Variable | Purpose |
|---|---|---|
| Web (Vercel) | `NEXT_PUBLIC_SUPABASE_URL` | Supabase project URL |
| Web (Vercel) | `NEXT_PUBLIC_SUPABASE_ANON_KEY` | Supabase anonymous/public key |
| Web (Vercel) | `SUPABASE_SERVICE_ROLE_KEY` | Supabase service role key (server-side only) |
| Web (Vercel) | `DATABASE_URL` | Supabase Postgres pooler connection string |
| Web (Vercel) | `UPSTASH_REDIS_REST_URL` | Redis connection (auto-provisioned) |
| Web (Vercel) | `UPSTASH_REDIS_REST_TOKEN` | Redis auth (auto-provisioned) |
| Web (Vercel) | `MEDPLUM_BASE_URL` | Medplum FHIR API base URL |
| Web (Vercel) | `MEDPLUM_CLIENT_ID` | Medplum OAuth client ID |
| Web (Vercel) | `MEDPLUM_CLIENT_SECRET` | Medplum OAuth client secret |
| Web (Vercel) | `ENGINE_GRPC_URL` | Rust engine gRPC endpoint |
| Web (Vercel) | `VERCEL_OIDC_TOKEN` | AI Gateway auth (auto-provisioned) |
| Engine | `DATABASE_URL` | Supabase Postgres connection string |
| Engine | `GRPC_PORT` | gRPC listen port (default 50051) |
| Engine | `AI_GATEWAY_URL` | AI Gateway endpoint for LLM steps |
| Engine | `AI_GATEWAY_API_KEY` | AI Gateway auth key (required — engine runs outside Vercel so OIDC is not available; use manual API key) |
| Engine | `MEDPLUM_BASE_URL` | Medplum FHIR API base URL |
| Engine | `MEDPLUM_CLIENT_ID` | Medplum OAuth client ID |
| Engine | `MEDPLUM_CLIENT_SECRET` | Medplum OAuth client secret |
| Engine | `REDIS_URL` | Upstash Redis for async signal queue |
| Engine | `PYTHON_SIDECAR_URL` | Python sidecar gRPC endpoint (default `unix:///tmp/python-sidecar.sock`) |
| Python sidecar | `GRPC_LISTEN` | Listen address (default `unix:///tmp/python-sidecar.sock`) |
| Python sidecar | `MAX_EXECUTION_TIME` | Per-step timeout in seconds (default 30) |

---

Concluding notes:

Using the above architecture, users can:

* Receive healthcare messages (HL7v2, FHIR, CSV, X12) via webhooks and have them automatically parsed, transformed, validated, and stored as FHIR resources in Medplum
* Configure Agents with multi-step workflows that combine deterministic code (Python), AI (LLM via AI Gateway), and FHIR operations
* Use AI-assisted data mapping to build and maintain transforms between healthcare data formats
* Monitor integration health, data quality, and execution performance through the analytics dashboard
* Trigger workflows from Medplum Subscriptions for event-driven clinical data processing
* Access everything from a web browser — no desktop app installation required

Production prerequisites (must be resolved before handling real patient data):

* PHI boundary enforcement — implement `phi_llm_policy` in agent policy, validate in engine before LLM dispatch
* BAA coverage for all PHI-touching infrastructure (Medplum, Fly.io/Railway/ECS, Supabase Postgres if storing clinical payloads)
* Decision on reference-only vs. inline PHI in signal payloads (impacts Supabase Postgres and Redis BAA requirements)
* Python sidecar sandboxing — cgroup limits, network egress policy, package allowlist, import restrictions
* Outbox publisher + reconciliation cron operational and tested
* Dead-letter signal handling in the dashboard
* MLLP listener tested against real HL7v2 feeds (if MLLP path is needed)
* Engine graceful shutdown tested under Fly.io/Railway deploy cycles

Advanced features scoped for later phases:

* Analytic database (OMOP CDM on top of Medplum's FHIR data)
* Horizontal scaling of the Rust engine (multiple instances consuming from Redis Streams, signal leasing via Postgres advisory locks)
* MCP Server implementation (expose Portico agents as MCP tools)
* Pre-configured Agent modules (HL7v2 ADT->FHIR, lab results pipeline, etc.)
* OpenTelemetry metrics + Grafana dashboards
* Multi-tenant support (multiple organizations, data isolation, per-tenant Medplum projects)
* SMART on FHIR app launcher (embed Portico in EHR workflows)
* HL7v2 de-identification step (for safe LLM processing of clinical narratives)

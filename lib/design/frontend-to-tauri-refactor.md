# Portico – Frontend ➜ Tauri 2.0 Command Migration

> Goal  Move security-sensitive data access and compute-heavy logic from SvelteKit (`app/src`) to Rust commands (`app/src-tauri`) for better encapsulation, offline support, and performance.

---

## 1  Inventory of Frontend Logic

| Area | Current implementation | Notes |
|------|------------------------|-------|
| **Supabase data access** | `app/src/lib/supabase.ts` + ad-hoc queries in:<br>• `routes/analytics/api.ts`<br>• `routes/agents/api.ts` (CRUD for agents, steps, runtime-sessions)<br>• Auth interactions in `+layout.svelte`, `login/+page.svelte`, `register/+page.svelte` | Keys (`VITE_SUPABASE_KEY`) shipped in bundle. Queries written in JS; some aggregate logic implemented client-side (e.g., grouping agent stats). |
| **Aggregate analytics** | `getAnalyticsCounts`, `getAgentPerformance`, `getStepPerformance`, `getErrorDistribution` in `analytics/api.ts` | Iterate over large result sets in JS, compute statistics. |
| **Navigation history store** | Lightweight store (`navigationStore.js`) mirrors to `sessionStorage`. | Pure UI concern; keep in front-end. |
| **Component-level computation** | `StepConfig.svelte` includes Python linting & CodeMirror editor. | UI/UX; remains in front-end. |
| **Auth session check** | `+layout.svelte` fetches supabase session and handles redirects. | Candidate for partial offload (see §4). |

---

## 2  Selection Criteria for Offloading to Rust Commands

A piece of logic is a good candidate when it:
1. Requires **secrets** (DB keys, API tokens) that shouldn't live in the JS bundle.
2. Performs **CPU-bound or bulk data processing** better handled natively.
3. Needs **privileged OS access** (filesystem, notifications, clipboard).
4. Should be available **offline** (by using embedded SQLite/cache rather than remote HTTP).

---

## 3  Recommended Candidates

### 3.1 Supabase CRUD & Aggregations

**Why:** Eliminates Supabase anon key from bundle, reduces number of HTTP round-trips, allows centralised caching and offline mode.

**Commands to create (examples):**
```rust
#[tauri::command]
async fn get_analytics_counts(...) -> Result<AnalyticsCounts, ApiError> { ... }
#[tauri::command]
async fn list_agents() -> Result<Vec<Agent>, ApiError> { ... }
#[tauri::command]
async fn save_agent(agent: AgentInput) -> Result<Vec<Agent>, ApiError> { ... }
// …similar for steps & runtime_sessions
```
Use `tauri-plugin-http` (or native `reqwest`) to call Supabase REST; store service URL & key in environment variables.

### 3.2 Analytics Computation

Move the grouping / average calculations now done in JS to Rust so that the front-end only requests finished stats.

Benefits:
- Heavy loops run natively → lower CPU usage in WebView.
- Easier unit-testing & benchmarking.

### 3.3 Auth Helper

Rather than using Supabase JS, expose a small Rust command:
```rust
#[tauri::command]
async fn login(email: String, password: String) -> Result<(), AuthError> { ... }
```
This lets us store refresh tokens securely via `tauri-plugin-store` and avoid `window.localStorage` leakage.

`+layout.svelte` would then call `invoke('login_status')` on startup.

### 3.4 Long-running Session Monitor (Future)

If runtime sessions generate logs or progress, create a streaming channel (`tauri::ipc::Channel`) emitting events to the UI instead of polling Supabase.

---

## 4  Stay in the Front-end

| Concern | Rationale |
|---------|-----------|
| **UI-only state (navigation store, component visibility, CodeMirror content)** | Pure UX; no benefit from moving. |
| **Python editor/linter in `StepConfig.svelte`** | Needs DOM; stays. |

---

## 5  Migration Steps

1. **Set up shared structs** in `src-tauri/gen` (or `serde`-derived structs) corresponding to Agent, Step, RuntimeSession, etc.
2. **Implement commands** in `src-tauri/src/commands.rs` and register them in `main.rs`.
3. Replace Supabase env variables in the front-end with Rust-side `dotenv` / `tauri::api::env` access.
4. **Write small JS wrappers** in `app/src/lib/api-client.ts` that call `invoke` and keep existing TS signatures—minimal UI diff.
5. Delete `$lib/supabase.ts` once all calls are ported.
6. Add `ENABLE_OFFLINE_MODE` flag to return fixture data when no network.

---

## 6  Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Increased compile times | Keep command module thin; reuse async functions across commands. |
| Rust ↔️ JS type drift | Generate TS bindings from Rust structs using `tauri-bindgen`. |
| Auth flow complexity | Start with email/password only; keep magic-link or OAuth on JS side until Rust plugin support is validated. |

---

## 7  Outcome Matrix

| Metric | Current | Post-migration |
|--------|---------|----------------|
| Supabase key exposure | In JS bundle | Rust env only |
| Requests per dashboard load | ~6 HTTP | 1 IPC |
| Offline usability | None | Basic (cached) |
| Front-end bundle size | 100% | 90-92% (remove supabase-js) |

---

> _Draft prepared Jun 2024 – outlines incremental extraction of backend-worthy logic from SvelteKit to Tauri 2.0 commands._

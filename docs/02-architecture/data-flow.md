# Data Flow

Below is the canonical sequence when a **Signal** of type `run` is created.

```mermaid
sequenceDiagram
  participant UI
  participant Supabase
  participant Bridge
  participant Engine
  participant DB as Postgres

  UI->>Supabase: INSERT signals (type='run', workflow_id, payload)
  Supabase-->>Bridge: Realtime INSERT event
  Bridge->>Engine: gRPC ProcessSignal(signal)
  Engine->>DB: SELECT workflow, steps
  Engine->>Engine: Execute steps (per-workflow queue)
  Engine->>DB: INSERT runtime_session + UPDATE signal
  Engine-->>Bridge: SignalResponse(success)
  Bridge-->>Supabase: UPDATE signals.status = 'complete'
```

## Other Signal Types

| Type | Purpose |
| ---- | ------- |
| `sync` | Forces Engine to rehydrate its in-memory state from the DB. |
| `fyi`  | Write-only, used for logging or metrics. No runtime execution. |

## Error Handling

* **Bridge** retries transient Supabase failures with exponential back-off (max 5 attempts).
* **Engine** returns a structured `Result` mapping to `signals.error`. Errors bubble back to the UI via Supabase Realtime.

## Workflow Queues & Concurrency

Each Workflow owns a FIFO queue inside Engine. A shared thread-pool drains the queues respecting ordering per Workflow while utilizing CPU cores efficiently.

```mermaid
graph LR
  subgraph Engine Thread Pool
    T1[Worker 1]
    T2[Worker 2]
    T3[Worker 3]
  end
  Q1[Workflow A Queue] --> T1
  Q1 --> T2
  Q2[Workflow B Queue] --> T3
```

## Agent Composition Flow

When an Agent needs to create a new Workflow:

```mermaid
sequenceDiagram
  participant UI
  participant Supabase
  participant Bridge
  participant Engine
  participant DB as Postgres

  UI->>Supabase: INSERT agent_composition_requests (agent_id, objective, context, constraints, is_ephemeral, auto_execute)
  Supabase-->>Bridge: Realtime INSERT event
  Bridge->>Engine: gRPC PlanWorkflow(request)
  Engine->>DB: SELECT agent (capabilities, policy)
  Engine->>Engine: Plan + Validate workflow spec
  Engine->>DB: INSERT workflow (created_by_agent_id)
  alt auto_execute && approved
    Engine->>DB: INSERT signal (type='run', workflow_id)
  end
  Engine-->>Bridge: PlanWorkflowResponse
```

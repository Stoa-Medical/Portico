# Data Flow

Below is the canonical sequence when a **Signal** of type `run` is created.

```mermaid
sequenceDiagram
  participant UI
  participant Supabase
  participant Bridge
  participant Engine
  participant DB as Postgres

  UI->>Supabase: INSERT signals (type='run', payload)
  Supabase-->>Bridge: Realtime INSERT event
  Bridge->>Engine: gRPC RunSignal(payload)
  Engine->>DB: SELECT agent, steps
  Engine->>Engine: Execute steps (thread pool)
  Engine->>DB: INSERT runtimesession + UPDATE signal
  Engine-->>Bridge: RunSignalResponse(success)
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

## Agent Queues & Concurrency

Each Agent owns a FIFO queue inside Engine. A shared thread-pool drains the queues respecting ordering per Agent while utilizing CPU cores efficiently.

```mermaid
graph LR
  subgraph Engine Thread Pool
    T1[Worker 1]
    T2[Worker 2]
    T3[Worker 3]
  end
  Q1[Agent A Queue] --> T1
  Q1 --> T2
  Q2[Agent B Queue] --> T3
```

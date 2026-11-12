# Portico Server

The backend that runs signals: a Rust engine, a Python sidecar for Python steps, and the Postgres schema both the engine and the web app use.

## How a signal runs

1. The web app inserts a row into `signals` with `status = 'pending'` and an `agent_id`.
2. The engine polls Postgres and leases the oldest pending signal whose agent has nothing in flight. Leasing sets `status = 'dispatched'` and a lease expiry.
3. For a `run` signal the engine opens a `runtime_sessions` row, loads the agent's steps in `step_order`, and runs them. Each step's output is the next step's input. The signal's `payload` is the first input.
4. The engine writes per-step results and timings to the session and settles the signal as `completed` (with `response_data`) or `failed` (with `error_message`). A run stops at the first failing step.
5. `sync` and `fyi` signals are acknowledged as `completed` without running steps.

Signals with no `agent_id` stay pending.

### Webhooks

Each inbound webhook URL names an integration: `POST /api/webhooks/{hl7,fhir,medplum}/<integration-id>`. The route copies the integration's `agent_id` onto the signal, so the integration decides which agent runs.

| Route | Integration `connection_type` | Retry key |
|-------|-------------------------------|-----------|
| `hl7/<id>` | `hl7v2-http` | MSH-10 control id |
| `fhir/<id>` | `rest-webhook` | resource type, id and `meta.versionId` |
| `medplum/<id>` | `fhir-subscription` | resource type, id and `meta.versionId` |

The sender authenticates with `Authorization: Bearer <secret>`. The integration stores only the secret's SHA-256, in `config.webhook_secret_sha256`. A retry with the same retry key returns the first signal with `"duplicate": true`. A message with no retry key always creates a new signal.

Responses: `404` for an unknown integration or one of another connection type, `401` for a missing or wrong secret, `409` for an integration that is inactive, outbound, or has no linked agent.

There is no UI for creating integrations yet. To add one:

```bash
SECRET=$(openssl rand -hex 32)
HASH=$(printf '%s' "$SECRET" | shasum -a 256 | cut -d' ' -f1)
psql "$DATABASE_URL" -c "
  INSERT INTO integrations (name, connection_type, direction, agent_id, config)
  VALUES ('Lab feed', 'hl7v2-http', 'inbound', '<agent-id>',
          jsonb_build_object('webhook_secret_sha256', '$HASH'))
  RETURNING id"
echo "Give the sender this secret: $SECRET"
```

### Ordering and recovery

- One signal per agent runs at a time, in arrival order. Up to `MAX_CONCURRENT_SIGNALS` agents run in parallel.
- This assumes a single engine instance. Two engines could lease two signals for the same agent at once.
- On startup and on shutdown the engine returns every `dispatched` signal to `pending` and marks its open session `cancelled`. A rerun executes the steps again from the start, so steps with side effects can repeat.
- A run is cut off 30 seconds before its lease expires and settles as `failed`.

## Components

### Engine (`engine/`)

| File | Role |
|------|------|
| `src/main.rs` | Config, poll loop, health server, graceful shutdown |
| `src/db.rs` | Every SQL statement the engine runs |
| `src/runner.rs` | Runs one leased signal to a settled state |
| `src/steps/` | One executor per step type |
| `src/services/fhir_client.rs` | Medplum OAuth and FHIR REST client |

Step types:

| Type | Status | Config |
|------|--------|--------|
| `python` | Works | `script`, optional `timeout_seconds`. The script reads `input_data` and assigns `output_data`. |
| `llm` | Calls the AI Gateway | `model`, `system_prompt`, optional `output_schema` |
| `fhir` | Calls Medplum | `operation` (`create`, `read`, `search`, `transaction`), `resource_type`, `id`, `params` |
| `transform` | Stub, returns its input | none |
| `validate` | Stub, always valid | none |

### Python sidecar (`python-sidecar/`)

FastAPI service with `POST /execute` and `GET /health`. It runs each script in a subprocess with a timeout and a restricted import hook. The import hook is not a security boundary. Isolation comes from the container.

### Schema (`database/schema.sql`)

The single schema definition. Atlas applies it declaratively. Compose applies it on startup through the `schema_migrator` service, which uses the throwaway `atlas_dev` database to compute the diff.

## Running

```bash
docker compose up --build
```

- Postgres on `:54322`
- Engine health on `:8080` (`GET /health`)
- Python sidecar on `:8001`

The web app connects to the same Postgres with `DATABASE_URL=postgresql://postgres:postgres@localhost:54322/postgres`.

## Verifying

```bash
./scripts/e2e.sh                                  # inserts signals directly
WEB_URL=http://localhost:3000 ./scripts/e2e.sh    # creates them through POST /api/signals
```

The script brings up the stack, seeds agents with a transform step and a Python step, and asserts on the rows the engine writes: a successful run, a run whose Python step raises, and a signal left `dispatched` across an engine restart. With `WEB_URL` set it also posts an HL7 message to a seeded integration and checks routing, retries, and the `401`, `404` and `409` responses.

To type-check the engine without a local Rust toolchain:

```bash
docker run --rm -v "$PWD/engine":/w -w /w rust:1.85 cargo check
```

## Environment variables

| Variable | Used by | Description |
|----------|---------|-------------|
| `DATABASE_URL` | Engine | Postgres connection string |
| `HEALTH_PORT` | Engine | Health check port (default 8080) |
| `PYTHON_SIDECAR_URL` | Engine | Sidecar endpoint (default `http://localhost:8001`) |
| `AI_GATEWAY_URL`, `AI_GATEWAY_API_KEY` | Engine | Required for `llm` steps |
| `MEDPLUM_BASE_URL`, `MEDPLUM_CLIENT_ID`, `MEDPLUM_CLIENT_SECRET` | Engine | Required for `fhir` steps |
| `POLL_INTERVAL_MS` | Engine | Idle poll interval (default 1000) |
| `MAX_CONCURRENT_SIGNALS` | Engine | Signals run at once (default 8) |
| `SIGNAL_LEASE_SECONDS` | Engine | Lease length (default 300) |
| `MAX_EXECUTION_TIME` | Sidecar | Upper bound on a Python step's timeout in seconds (default 60) |

## Not implemented

The design in [`design/2-cloud-native.md`](../design/2-cloud-native.md) also describes gRPC dispatch, Redis Streams, a Medplum outbox, an MLLP listener, retries and dead-letter handling. None of those exist in this tree. The `outbox_events`, `dead_letter_signals`, `engine_state` and `audit_log` tables are in the schema and nothing writes to them yet.

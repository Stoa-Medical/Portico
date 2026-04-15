-- Portico operational schema (Atlas SQL — single source of truth)

-- Enums

CREATE TYPE step_type AS ENUM ('python', 'llm', 'transform', 'validate', 'fhir');
CREATE TYPE signal_type AS ENUM ('run', 'sync', 'fyi');
CREATE TYPE signal_status AS ENUM ('pending', 'dispatched', 'completed', 'failed');
CREATE TYPE running_status AS ENUM ('waiting', 'running', 'completed', 'failed', 'cancelled');
CREATE TYPE connection_type AS ENUM ('hl7v2-mllp', 'hl7v2-http', 'fhir-subscription', 'rest-webhook', 'file-drop');
CREATE TYPE connection_direction AS ENUM ('inbound', 'outbound');
CREATE TYPE integration_status AS ENUM ('active', 'inactive', 'error');

-- Tables (no foreign keys)

CREATE TABLE agents (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  description TEXT,
  capabilities JSONB,
  policy JSONB,
  preferred_model TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE engine_state (
  id INTEGER PRIMARY KEY DEFAULT 1,
  last_redis_stream_id TEXT,
  last_heartbeat TIMESTAMPTZ,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE audit_log (
  id SERIAL PRIMARY KEY,
  actor TEXT NOT NULL,
  action TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  resource_id UUID,
  details JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tables (FK → agents)

CREATE TABLE steps (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  agent_id UUID REFERENCES agents(id),
  name TEXT NOT NULL,
  step_type step_type NOT NULL,
  step_order INTEGER NOT NULL,
  config JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE runtime_sessions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  agent_id UUID REFERENCES agents(id),
  signal_id UUID,
  status running_status NOT NULL DEFAULT 'waiting',
  step_results JSONB,
  step_execution_ms JSONB,
  total_execution_ms INTEGER,
  data_quality_score REAL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  completed_at TIMESTAMPTZ
);

CREATE TABLE data_mappings (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  source_schema JSONB,
  target_schema JSONB,
  field_mappings JSONB,
  ai_generated BOOLEAN DEFAULT false,
  agent_id UUID REFERENCES agents(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE integrations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  connection_type connection_type NOT NULL,
  direction connection_direction NOT NULL,
  config JSONB,
  agent_id UUID REFERENCES agents(id),
  status integration_status NOT NULL DEFAULT 'active',
  last_seen_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tables (FK → agents, runtime_sessions)

CREATE TABLE signals (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  signal_type signal_type NOT NULL,
  source TEXT NOT NULL,
  idempotency_key TEXT NOT NULL UNIQUE,
  agent_id UUID REFERENCES agents(id),
  payload JSONB,
  response_data JSONB,
  runtime_session_id UUID REFERENCES runtime_sessions(id),
  source_metadata JSONB,
  status signal_status NOT NULL DEFAULT 'pending',
  error_message TEXT,
  leased_at TIMESTAMPTZ,
  lease_expires_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tables (FK → signals, agents)

CREATE TABLE dead_letter_signals (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  signal_id UUID REFERENCES signals(id),
  agent_id UUID REFERENCES agents(id),
  failure_reason TEXT,
  last_step_index INTEGER,
  last_error JSONB,
  original_payload JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  resolved_at TIMESTAMPTZ,
  resolved_by TEXT
);

CREATE TABLE outbox_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  signal_id UUID REFERENCES signals(id),
  step_index INTEGER,
  event_type TEXT NOT NULL,
  payload JSONB,
  status TEXT NOT NULL DEFAULT 'pending',
  medplum_response JSONB,
  retry_count INTEGER DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  sent_at TIMESTAMPTZ
);

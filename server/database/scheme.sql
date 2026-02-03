-- ============================================
-- CONNECTORS
-- ============================================

CREATE TYPE connector_type AS ENUM ('fhir', 'http', 'webhook', 'database');

CREATE TYPE connector_status AS ENUM ('active', 'paused', 'error', 'initializing');

CREATE TABLE connectors (
    id int GENERATED ALWAYS AS IDENTITY,
    global_uuid uuid DEFAULT gen_random_uuid(),
    name varchar(255) NOT NULL,
    connector_type connector_type NOT NULL,
    config jsonb DEFAULT '{}'::jsonb, -- endpoint, auth, polling_interval, resources
    schema_mapping jsonb DEFAULT '{}'::jsonb, -- source_field -> target_field rules (JSONPath)
    status connector_status DEFAULT 'initializing',
    last_sync_at timestamptz,
    last_error text,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    PRIMARY KEY (id)
);

-- ============================================
-- RECIPES
-- ============================================

CREATE TABLE recipes (
    id int GENERATED ALWAYS AS IDENTITY,
    global_uuid uuid DEFAULT gen_random_uuid(),
    name varchar(255) NOT NULL,
    slug varchar(100) NOT NULL, -- e.g. prior_auth, referral_tracker
    description text,
    workflow_template jsonb NOT NULL, -- Pre-configured workflow + steps definition
    required_connectors varchar(50)[] DEFAULT '{}', -- e.g. {fhir, http}
    config_schema jsonb DEFAULT '{}'::jsonb, -- JSON Schema for user-provided config
    is_builtin boolean DEFAULT true,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX idx_recipes_slug ON recipes (slug);

-- ============================================
-- ANALYTICS FACT TABLES
-- ============================================

CREATE TABLE fact_connector_events (
    id bigint GENERATED ALWAYS AS IDENTITY,
    connector_id int NOT NULL,
    resource_type varchar(100) NOT NULL,
    action varchar(50) NOT NULL, -- created, updated, deleted
    resource_id varchar(255) NOT NULL,
    data_hash varchar(64), -- SHA256 for dedup
    event_at timestamptz DEFAULT now(),
    PRIMARY KEY (id),
    CONSTRAINT fk_connector FOREIGN KEY (connector_id) REFERENCES connectors (id) ON DELETE CASCADE
);

CREATE INDEX idx_connector_events_time ON fact_connector_events (connector_id, event_at);

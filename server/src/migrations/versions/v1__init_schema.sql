-- V1__init_schema.sql
BEGIN;

-- Metadata tracking
CREATE TABLE IF NOT EXISTS schema_migrations (
    version int PRIMARY KEY,
    applied_at timestamp with time zone DEFAULT current_timestamp
);

-- Your actual schema changes
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name text NOT NULL
);

-- Record this migration
INSERT INTO schema_migrations (version) VALUES (1);

COMMIT;

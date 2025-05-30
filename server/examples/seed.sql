-- Supabase seed script for development data
--
-- INSTRUCTIONS TO RUN:
-- 1. Make sure Supabase is running: `supabase start` (from the server directory)
-- 2. Run this seed script: `psql postgresql://postgres:postgres@localhost:54322/postgres -f server/examples/seed.sql`
-- 3. For additional runtime session data, use: `cd server/examples && python scripts/generate_seed_data.py`
--
-- TO RESET DATABASE:
-- Option 1: `supabase db reset` (from the server directory)
-- Option 2: `cd server && ./reset_db.sh`
--
-- IMPORTANT NOTE ABOUT ANALYTICS:
-- The analytics page filters data by owner_id. If your agents table has an owner_id column
-- (likely added by Supabase auth), you'll need to either:
-- 1. Update the seed data to include your user ID in the owner_id field
-- 2. Or modify the analytics queries to show all data for demo purposes

-- Clear existing data first
TRUNCATE TABLE runtime_sessions, steps, signals, agents RESTART IDENTITY CASCADE;

-- Insert sample agents (simplified for demo)
-- Note: If using Supabase auth, you may need to add owner_id column
-- Check if owner_id column exists and insert accordingly
DO $$
BEGIN
    -- Check if owner_id column exists
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'agents' AND column_name = 'owner_id') THEN
        -- Insert with a demo user ID (you'll need to replace with an actual user ID)
        INSERT INTO agents (name, type, description, agent_state, agent_name, agent_type, owner_id)
        VALUES
          ('Hacker News Scraper', 'information', 'Scrapes Hacker News and summarizes trending topics', 'stable', 'hacker_news_scraper', 'content', null),
          ('HL7 to FHIR Converter', 'integration', 'Converts HL7 messages to FHIR resources', 'stable', 'hl7_to_fhir_agent', 'transform', null);
    ELSE
        -- Insert without owner_id
        INSERT INTO agents (name, type, description, agent_state, agent_name, agent_type)
        VALUES
          ('Hacker News Scraper', 'information', 'Scrapes Hacker News and summarizes trending topics', 'stable', 'hacker_news_scraper', 'content'),
          ('HL7 to FHIR Converter', 'integration', 'Converts HL7 messages to FHIR resources', 'stable', 'hl7_to_fhir_agent', 'transform');
    END IF;
END $$;

-- Insert sample steps for Hacker News Scraper
INSERT INTO steps (agent_id, name, description, step_type, step_content)
VALUES
  -- Hacker News Scraper steps
  (1, 'Scrape Hacker News', 'Fetches top stories from Hacker News', 'webscrape', 'https://news.ycombinator.com'),
  (1, 'Summarize News', 'Summarizes the scraped content', 'prompt', 'Summarize the top technology news from Hacker News in 3 bullet points.'),
  (1, 'Save Summary', 'Saves the summary to a file', 'python', E'import json\n\n# Get summary from previous step\nsummary = source.get("response", "No summary generated")\n\n# Save to file\nresult = {\n    "summary": summary,\n    "timestamp": "2025-05-30",\n    "status": "success"\n}'),

  -- HL7 to FHIR Converter steps
  (2, 'Parse HL7', 'Parses HL7 message into JSON', 'python', E'# Simple HL7 parser\nhl7_data = source.get("raw_hl7", "MSH|^~\\\\&|ADT")\n\n# Extract message type\nparts = hl7_data.split("|")\n\nresult = {\n    "status": "success",\n    "message_type": parts[0] if parts else "Unknown",\n    "parsed_data": {"type": "ADT", "patient_id": "12345"}\n}'),
  (2, 'Convert to FHIR', 'Maps HL7 data to FHIR format', 'prompt', 'Convert this HL7 data to a FHIR Patient resource.'),
  (2, 'Store FHIR', 'Stores the FHIR resource', 'python', E'# Store FHIR resource\nfhir_data = source.get("response", "{}")\n\nresult = {\n    "status": "success",\n    "resource_type": "Patient",\n    "resource_id": "pat-12345",\n    "stored": True\n}');

-- Update agent step_ids array
UPDATE agents
SET step_ids = (SELECT ARRAY_AGG(id ORDER BY id) FROM steps WHERE agent_id = agents.id)
WHERE agent_name IN ('hacker_news_scraper', 'hl7_to_fhir_agent');

-- Insert sample Signals
INSERT INTO signals (agent_id, user_requested_uuid, signal_type, initial_data) VALUES
  (1, gen_random_uuid(), 'run', '{}'),
  (2, gen_random_uuid(), 'run', '{"raw_hl7": "MSH|^~&|ADT|HOSPITAL|EPIC|HOSPITAL|20250530120000||ADT^A01|12345|P|2.5"}');

-- Insert sample Runtime Sessions (simplified)
-- Wait for agents to be inserted first
DO $$
DECLARE
    agent1_id INTEGER;
    agent2_id INTEGER;
BEGIN
    -- Get agent IDs
    SELECT id INTO agent1_id FROM agents WHERE agent_name = 'hacker_news_scraper';
    SELECT id INTO agent2_id FROM agents WHERE agent_name = 'hl7_to_fhir_agent';

    -- Insert runtime sessions only if agents exist
    IF agent1_id IS NOT NULL THEN
        INSERT INTO runtime_sessions (
            global_uuid, created_at, updated_at, requested_by_agent_id,
            rts_status, initial_data, latest_step_idx, latest_result,
            step_execution_times, total_execution_time
        ) VALUES (
            gen_random_uuid(),
            NOW() - INTERVAL '1 hour',
            NOW() - INTERVAL '59 minutes',
            agent1_id,
            'completed',
            '{}',
            2,
            '{"status":"success","summary":"Tech news summarized"}',
            ARRAY[1.5, 2.3, 0.8],
            4.6
        );
    END IF;

    IF agent2_id IS NOT NULL THEN
        INSERT INTO runtime_sessions (
            global_uuid, created_at, updated_at, requested_by_agent_id,
            rts_status, initial_data, latest_step_idx, latest_result,
            step_execution_times, total_execution_time
        ) VALUES (
            gen_random_uuid(),
            NOW() - INTERVAL '30 minutes',
            NOW() - INTERVAL '28 minutes',
            agent2_id,
            'completed',
            '{"raw_hl7": "MSH|^~&|ADT"}',
            2,
            '{"status":"success","resource_type":"Patient"}',
            ARRAY[0.5, 3.2, 0.3],
            4.0
        );
    END IF;
END $$;

-- Confirm the seeded data
SELECT 'Agents: ' || COUNT(*) FROM agents;
SELECT 'Steps: ' || COUNT(*) FROM steps;
SELECT 'Signals: ' || COUNT(*) FROM signals;
SELECT 'Runtime Sessions: ' || COUNT(*) FROM runtime_sessions;

-- Add Supabase realtime tables
ALTER PUBLICATION supabase_realtime ADD TABLE signals;
ALTER PUBLICATION supabase_realtime ADD TABLE agents;

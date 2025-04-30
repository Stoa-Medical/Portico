-- SQL script to create a new Signal that triggers the Hacker News scraper agent
-- This file demonstrates how to create a signal that will trigger the agent to run

-- First, let's make sure we have the correct agent ID for the Hacker News Scraper
-- In the seed.sql file, it's agent_id = 4, but let's verify with a query
-- Uncomment the following line if you need to verify the agent ID:
-- SELECT id, name FROM agents WHERE name = 'Hacker News Scraper';

-- Create a new signal to trigger the Hacker News scraper agent
INSERT INTO signals (
    agent_id,
    user_requested_uuid,
    signal_type,
    initial_data
)
VALUES (
    4, -- Agent ID for Hacker News Scraper (adjust if different in your database)
    gen_random_uuid(), -- Generate a random UUID for this request
    'run', -- Signal type 'run' to execute the agent
    '{"date": "2025-04-30", "custom_output_path": "/tmp/hacker_news_custom.json"}' -- Initial data with today's date
);

-- Verify that the signal was created
SELECT
    s.id,
    a.name as agent_name,
    s.signal_type,
    s.created_at,
    s.initial_data
FROM
    signals s
JOIN
    agents a ON s.agent_id = a.id
WHERE
    a.name = 'Hacker News Scraper'
ORDER BY
    s.created_at DESC
LIMIT 1;

-- Note: After running this signal, you can check for runtime sessions created for this agent:
-- SELECT * FROM runtime_sessions WHERE requested_by_agent_id = 4 ORDER BY created_at DESC LIMIT 1;

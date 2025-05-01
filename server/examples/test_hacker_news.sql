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
    1, -- Agent ID for Hacker News Scraper (updated to match actual ID in database)
    gen_random_uuid(), -- Generate a random UUID for this request
    'run', -- Signal type 'run' to execute the agent
    '{"custom_output_path": "/tmp/hacker_news_custom.json"}' -- Custom output path for results
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

-- After running this test, you can check if a runtime session was created:
-- SELECT * FROM runtime_sessions WHERE requested_by_agent_id = 1 ORDER BY created_at DESC LIMIT 1;

-- And you can check if the file was created (you would need to do this manually on the server):
-- The file should be located at: /tmp/hacker_news_custom.json

-- The workflow of this test:
-- 1. Signal triggers the Hacker News Scraper agent to run
-- 2. The agent's steps will:
--    a. Scrape Hacker News website (via webscrape step)
--    b. Save the scraped data to a JSON file (via Python step)
--    c. Summarize the news using an LLM (via prompt step)
--    d. Append the summary to the JSON file (via Python step)
-- 3. The final result will be stored in the runtime_sessions table

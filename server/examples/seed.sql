-- Supabase seed script for development data
-- Load with: psql postgresql://postgres:postgres@localhost:54322/postgres -f seed.sql
-- Reset ENTIRE database with `supabase db reset` (be careful! Meant for development only)

-- Clear existing data first
TRUNCATE TABLE runtime_sessions, steps, signals, agents RESTART IDENTITY CASCADE;

-- Insert sample agents
INSERT INTO agents (name, type, description, agent_state, agent_name, agent_type)
VALUES
  ('Hacker News Scraper', 'information', 'Scrapes Hacker News, saves the data, summarizes it with LLM, and appends the summary to the file', 'stable', 'hacker_news_scraper', 'content');

-- Insert sample steps for each agent
INSERT INTO steps (agent_id, name, description, step_type, step_content)
VALUES
  -- Hacker News Scraper steps
  (1, 'Scrape Hacker News', 'Grabs information from news.ycombinator.com and identifies main topics', 'webscrape', 'https://news.ycombinator.com'),
  (1, 'Save News Data', 'Saves the scraped news data to a file', 'python', E'import json\nimport os\n\n# Get the scraped data from the previous step\nnews_data = source.get("data", source)\n\n# Define the output file path - check both globals and source\ncustom_output_path = None\n# In Python steps, the RUN signal\'s initial_data is accessible directly via a global variable \'data\'\nif \'data\' in globals() and isinstance(data, dict) and "custom_output_path" in data:\n    custom_output_path = data["custom_output_path"]\n\n# Check the source for custom_output_path as a fallback\nif not custom_output_path and isinstance(source, dict) and "custom_output_path" in source:\n    custom_output_path = source["custom_output_path"]\n\n# Use default if no custom path was found\noutput_file = custom_output_path if custom_output_path else "/tmp/hacker_news_data.json"\n\ntry:\n    # Create directory if it doesn\'t exist\n    os.makedirs(os.path.dirname(output_file), exist_ok=True)\n    \n    # Save the data to a file\n    with open(output_file, "w") as f:\n        json.dump(news_data, f, indent=2)\n    \n    result = {\n        "file_path": output_file,\n        "item_count": len(news_data) if isinstance(news_data, list) else 1,\n        "status": "success"\n    }\nexcept Exception as e:\n    result = {\n        "error": str(e),\n        "status": "error"\n    }'),
  (1, 'Summarize News', 'Generates a summary of the key updates from Hacker News', 'prompt', 'You are a news analyst specializing in technology trends.\n\nI have scraped the top stories from Hacker News. Please analyze the data and provide a concise summary of the key updates and trends. Focus on identifying the main topics, any emerging patterns, and highlight the most significant stories based on points and discussion activity.\n\nFormat your response as a brief executive summary that could be shared with a technology team.'),
  (1, 'Append Summary', 'Appends the generated summary to the news data file', 'python', E'import json\nimport os\nimport glob\n\n# Get the summary from the previous step - using the standardized format\nsummary = source.get("response", "No summary generated")\n\n# We need to find the file path, but we don\'t have access to previous_results\n# Instead, use a hardcoded path or look for recently created files\nfile_path = "/tmp/hacker_news_data.json"\n\n# Check if the file exists before proceeding\nif not os.path.exists(file_path):\n    # Try to find the most recent json file in /tmp\n    json_files = glob.glob("/tmp/hacker_news*.json")\n    if json_files:\n        # Sort by creation time and get the most recent one\n        file_path = max(json_files, key=os.path.getctime)\n\n# In Python steps, the RUN signal\'s initial_data is accessible directly via a global variable \'data\'\n# so we can check if data and custom_output_path are defined\ntry:\n    # Custom output path might be in \'data\' if available from signal\n    custom_output_path = None\n    if \'data\' in globals() and isinstance(data, dict) and "custom_output_path" in data:\n        custom_output_path = data["custom_output_path"]\n    \n    # Check the source for custom_output_path as a fallback\n    if not custom_output_path and isinstance(source, dict) and "custom_output_path" in source:\n        custom_output_path = source["custom_output_path"]\n\n    # Load the existing data\n    with open(file_path, "r") as f:\n        data = json.load(f)\n    \n    # Create a new file with both data and summary\n    output_file = custom_output_path if custom_output_path else "/tmp/hacker_news_report.json"\n    # Create directory if it doesn\'t exist\n    os.makedirs(os.path.dirname(output_file), exist_ok=True)\n    \n    with open(output_file, "w") as f:\n        json.dump({\n            "data": data,\n            "summary": summary\n        }, f, indent=2)\n    \n    result = {\n        "original_file": file_path,\n        "report_file": output_file,\n        "status": "success"\n    }\nexcept Exception as e:\n    result = {\n        "error": str(e),\n        "status": "error"\n    }');

-- Update agent step_ids array
UPDATE agents
SET step_ids = ARRAY[1, 2, 3, 4]
WHERE id = 1;

-- Confirm the seeded data
SELECT 'Agents: ' || COUNT(*) FROM agents;
SELECT 'Steps: ' || COUNT(*) FROM steps;
SELECT 'Signals: ' || COUNT(*) FROM signals;
SELECT 'Runtime Sessions: ' || COUNT(*) FROM runtime_sessions;

-- Add Supabase realtime tables
ALTER PUBLICATION supabase_realtime ADD TABLE signals;
ALTER PUBLICATION supabase_realtime ADD TABLE agents;

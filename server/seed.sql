-- Supabase seed script for development data
-- Load with: psql postgresql://postgres:postgres@localhost:54322/postgres -f seed.sql
-- Reset ENTIRE database with `supabase db reset` (be careful! Meant for development only)

-- Clear existing data first
TRUNCATE TABLE runtime_sessions, steps, signals, agents RESTART IDENTITY CASCADE;

-- Insert sample agents
INSERT INTO agents (name, type, description, agent_state, agent_name, agent_type)
VALUES
  ('File Processor', 'automation', 'Processes and analyzes file content', 'stable', 'file_processor', 'utility'),
  ('Email Notifier', 'notification', 'Sends email notifications', 'stable', 'email_notifier', 'communication'),
  ('Data Analyzer', 'analysis', 'Performs statistical analysis on data sets', 'unstable', 'data_analyzer', 'analytics'),
  ('Hacker News Scraper', 'information', 'Scrapes Hacker News, saves the data, summarizes it with LLM, and appends the summary to the file', 'stable', 'hacker_news_scraper', 'content');

-- Insert sample steps for each agent
INSERT INTO steps (agent_id, name, description, step_type, step_content)
VALUES
  -- File Processor steps
  (1, 'Read File', 'Reads the content of the input file', 'python', E'file_path = source.get("file_path", "")\nif file_path:\n    try:\n        with open(file_path, "r") as f:\n            content = f.read()\n        result = {"content": content, "status": "success"}\n    except Exception as e:\n        result = {"error": str(e), "status": "error"}\nelse:\n    result = {"error": "No file path provided", "status": "error"}'),
  (1, 'Parse Content', 'Parses the file content into structured data', 'python', E'content = source.get("content", "")\n# Parse the content based on its structure\nif content:\n    # Simple example: Split by lines and count\n    lines = content.split("\\n")\n    result = {\n        "parsed": content,\n        "line_count": len(lines),\n        "status": "success"\n    }\nelse:\n    result = {"error": "No content to parse", "status": "error"}'),
  (1, 'Generate Report', 'Creates a summary report of the file', 'prompt', 'Create a summary report of the parsed data (attached)'),

  -- Email Notifier steps
  (2, 'Format Message', 'Formats the notification message', 'prompt', 'Create an email message for {{recipient}} about {{subject}}'),
  (2, 'Send Email', 'Sends the formatted email', 'python', E'recipient = source.get("recipient", "")\nsubject = source.get("subject", "")\nbody = source.get("response", "")\n\nif not recipient or not subject or not body:\n    result = {\n        "error": "Missing required fields",\n        "status": "error"\n    }\nelse:\n    # In a real implementation, this would connect to an email service\n    # This is a simulation for the example\n    result = {\n        "status": "sent",\n        "to": recipient,\n        "subject": subject,\n        "message_length": len(body)\n    }'),

  -- Data Analyzer steps
  (3, 'Load Dataset', 'Loads the dataset from the specified source', 'python', E'dataset_source = source.get("source", "")\n\n# In a real scenario, we would load from the source\n# For this example, we create mock data\nif dataset_source:\n    # Simulate loading different datasets based on source\n    if "sales" in dataset_source:\n        data = [10, 25, 30, 15, 20]\n    elif "users" in dataset_source:\n        data = [100, 150, 120, 200, 180]\n    else:\n        data = [1, 2, 3, 4, 5]\n    \n    result = {\n        "data": data,\n        "source": dataset_source,\n        "status": "success"\n    }\nelse:\n    result = {\n        "data": [1, 2, 3, 4, 5],  # Default dataset\n        "source": "default",\n        "status": "warning"\n    }'),
  (3, 'Run Analysis', 'Performs statistical analysis on the dataset', 'python', E'dataset = source.get("data", [])\n\nif not dataset:\n    result = {"error": "No dataset provided", "status": "error"}\n    return result\n\ntry:\n    # Calculate basic statistics\n    mean_value = sum(dataset) / len(dataset)\n    min_value = min(dataset)\n    max_value = max(dataset)\n    \n    result = {\n        "mean": mean_value,\n        "min": min_value,\n        "max": max_value,\n        "count": len(dataset),\n        "source": source.get("source", "unknown"),\n        "status": "success"\n    }\nexcept Exception as e:\n    result = {"error": str(e), "status": "error"}'),
  (3, 'Visualize Results', 'Creates visualizations of the analysis results', 'python', E'analysis_results = source\n\n# Check if we have valid analysis results\nif "mean" not in analysis_results:\n    result = {"error": "Missing analysis data", "status": "error"}\n    return result\n\n# In a real scenario, we would generate actual visualization data\n# Here we just prepare the data structure for visualization\nresult = {\n    "chart_data": {\n        "labels": ["Mean", "Min", "Max"],\n        "values": [\n            analysis_results.get("mean", 0),\n            analysis_results.get("min", 0),\n            analysis_results.get("max", 0)\n        ]\n    },\n    "visualization_type": "bar_chart",\n    "title": f"Analysis of {analysis_results.get(\'source\', \'unknown\')} Dataset",\n    "status": "success"\n}'),

  -- Hacker News Scraper steps
  (4, 'Scrape Hacker News', 'Grabs information from news.ycombinator.com and identifies main topics', 'webscrape', 'https://news.ycombinator.com'),
  (4, 'Save News Data', 'Saves the scraped news data to a file', 'python', E'import json\nimport os\n\n# Get the scraped data from the previous step\nnews_data = source\n\n# Define the output file path\ncustom_output_path = source.get("custom_output_path", None)\noutput_file = custom_output_path if custom_output_path else "/tmp/hacker_news_data.json"\n\ntry:\n    # Create directory if it doesn\'t exist\n    os.makedirs(os.path.dirname(output_file), exist_ok=True)\n    \n    # Save the data to a file\n    with open(output_file, "w") as f:\n        json.dump(news_data, f, indent=2)\n    \n    result = {\n        "file_path": output_file,\n        "item_count": len(news_data) if isinstance(news_data, list) else 1,\n        "status": "success"\n    }\nexcept Exception as e:\n    result = {\n        "error": str(e),\n        "status": "error"\n    }'),
  (4, 'Summarize News', 'Generates a summary of the key updates from Hacker News', 'prompt', 'You are a news analyst specializing in technology trends.\n\nI have scraped the top stories from Hacker News. Please analyze the data and provide a concise summary of the key updates and trends. Focus on identifying the main topics, any emerging patterns, and highlight the most significant stories based on points and discussion activity.\n\nFormat your response as a brief executive summary that could be shared with a technology team.'),
  (4, 'Append Summary', 'Appends the generated summary to the news data file', 'python', E'import json\n\n# Get the summary from the previous step\nsummary = source.get("response", "No summary generated")\n\n# Get the file path from step 2\nfile_path = source.get("file_path", "/tmp/hacker_news_data.json")\n\n# Check if a custom output path was provided in the initial data\ncustom_output_path = source.get("custom_output_path", None)\n\ntry:\n    # Load the existing data\n    with open(file_path, "r") as f:\n        data = json.load(f)\n    \n    # Create a new file with both data and summary\n    output_file = custom_output_path if custom_output_path else "/tmp/hacker_news_report.json"\n    with open(output_file, "w") as f:\n        json.dump({\n            "data": data,\n            "summary": summary\n        }, f, indent=2)\n    \n    result = {\n        "original_file": file_path,\n        "report_file": output_file,\n        "status": "success"\n    }\nexcept Exception as e:\n    result = {\n        "error": str(e),\n        "status": "error"\n    }');

-- Update agent step_ids array
UPDATE agents
SET step_ids = ARRAY[1, 2, 3]
WHERE id = 1;

UPDATE agents
SET step_ids = ARRAY[4, 5]
WHERE id = 2;

UPDATE agents
SET step_ids = ARRAY[6, 7, 8]
WHERE id = 3;

UPDATE agents
SET step_ids = ARRAY[9, 10, 11, 12]
WHERE id = 4;

-- Insert sample runtime sessions
INSERT INTO runtime_sessions (requested_by_agent_id, rts_status, initial_data, latest_step_idx, latest_result, step_execution_times, step_ids, total_execution_time)
VALUES
  (1, 'completed', '{"file_path": "/tmp/doc.pdf"}', 3, '{"summary": "Processed 42 items"}', ARRAY[0.123, 0.456, 0.789], ARRAY[1, 2, 3], 1.368),
  (2, 'completed', '{"recipient": "user@example.com", "subject": "Weekly Report"}', 2, '{"status": "sent"}', ARRAY[0.234, 0.567], ARRAY[4, 5], 0.801),
  (3, 'running', '{"dataset_id": "ds-456"}', 1, '{"data": [10, 20, 30, 40, 50]}', ARRAY[0.345], ARRAY[6], 0.345),
  (4, 'completed', '{"date": "2025-04-30"}', 4, '{"report_file": "/tmp/hacker_news_report.json", "status": "success"}', ARRAY[1.234, 0.345, 2.567, 0.456], ARRAY[9, 10, 11, 12], 4.602);

-- Insert sample signals (after runtime_sessions to satisfy foreign key constraint)
INSERT INTO signals (agent_id, user_requested_uuid, signal_type, rts_id, initial_data, response_data)
VALUES
  (1, gen_random_uuid(), 'run', 1, '{"file_path": "/tmp/sample.txt"}', '{"report": "File processed successfully"}'),
  (2, gen_random_uuid(), 'fyi', 2, '{"message": "System update completed"}', NULL),
  (3, gen_random_uuid(), 'sync', 3, '{"dataset_id": "ds-123"}', NULL),
  (4, gen_random_uuid(), 'run', 4, '{"date": "2025-04-30"}', '{"status": "News data successfully scraped, summarized, and saved to file"}');

-- Confirm the seeded data
SELECT 'Agents: ' || COUNT(*) FROM agents;
SELECT 'Steps: ' || COUNT(*) FROM steps;
SELECT 'Signals: ' || COUNT(*) FROM signals;
SELECT 'Runtime Sessions: ' || COUNT(*) FROM runtime_sessions;

-- Add Supabase realtime tables
ALTER PUBLICATION supabase_realtime ADD TABLE signals;
ALTER PUBLICATION supabase_realtime ADD TABLE agents;

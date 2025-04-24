-- Supabase seed script for development data
-- Run with: psql postgresql://postgres:postgres@localhost:54322/postgres -f seed.sql

-- Clear existing data first
TRUNCATE TABLE runtime_sessions, steps, signals, agents RESTART IDENTITY CASCADE;

-- Insert sample agents
INSERT INTO agents (name, type, description, agent_state, agent_name, agent_type)
VALUES
  ('File Processor', 'automation', 'Processes and analyzes file content', 'stable', 'file_processor', 'utility'),
  ('Email Notifier', 'notification', 'Sends email notifications', 'stable', 'email_notifier', 'communication'),
  ('Data Analyzer', 'analysis', 'Performs statistical analysis on data sets', 'unstable', 'data_analyzer', 'analytics');

-- Insert sample steps for each agent
INSERT INTO steps (agent_id, name, description, step_type, step_content)
VALUES
  -- File Processor steps
  (1, 'Read File', 'Reads the content of the input file', 'python', 'def read_file(file_path):\n    with open(file_path, "r") as f:\n        return f.read()'),
  (1, 'Parse Content', 'Parses the file content into structured data', 'python', 'def parse_content(content):\n    # Parsing logic here\n    return {"parsed": content}'),
  (1, 'Generate Report', 'Creates a summary report of the file', 'prompt', 'Create a summary report of the parsed data (attached)'),

  -- Email Notifier steps
  (2, 'Format Message', 'Formats the notification message', 'prompt', 'Create an email message for {{recipient}} about {{subject}}'),
  (2, 'Send Email', 'Sends the formatted email', 'python', 'def send_email(recipient, subject, body):\n    # Email sending logic\n    return {"status": "sent"}'),

  -- Data Analyzer steps
  (3, 'Load Dataset', 'Loads the dataset from the specified source', 'python', 'def load_dataset(source):\n    # Loading logic\n    return {"data": [1, 2, 3, 4, 5]}'),
  (3, 'Run Analysis', 'Performs statistical analysis on the dataset', 'python', 'def analyze(dataset):\n    # Analysis logic\n    return {"mean": sum(dataset["data"]) / len(dataset["data"])}'),
  (3, 'Visualize Results', 'Creates visualizations of the analysis results', 'python', 'def visualize(results):\n    # Visualization logic\n    return {"chart_data": results}');

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

-- Insert sample signals
INSERT INTO signals (agent_id, user_requested_uuid, signal_type, signal_status, initial_data, response_data)
VALUES
  (1, gen_random_uuid(), 'command', 'completed', '{"file_path": "/tmp/sample.txt"}', '{"report": "File processed successfully"}'),
  (2, gen_random_uuid(), 'fyi', 'completed', '{"message": "System update completed"}', NULL),
  (3, gen_random_uuid(), 'sync', 'running', '{"dataset_id": "ds-123"}', NULL),
  (NULL, gen_random_uuid(), 'command', 'waiting', '{"action": "restart_service"}', NULL);

-- Insert sample runtime sessions
INSERT INTO runtime_sessions (requested_by_agent_id, rts_status, initial_data, latest_step_idx, latest_result, step_execution_times, step_ids, total_execution_time)
VALUES
  (1, 'completed', '{"file_path": "/tmp/doc.pdf"}', 3, '{"summary": "Processed 42 items"}', ARRAY[0.123, 0.456, 0.789], ARRAY[1, 2, 3], 1.368),
  (2, 'completed', '{"recipient": "user@example.com", "subject": "Weekly Report"}', 2, '{"status": "sent"}', ARRAY[0.234, 0.567], ARRAY[4, 5], 0.801),
  (3, 'running', '{"dataset_id": "ds-456"}', 1, '{"data": [10, 20, 30, 40, 50]}', ARRAY[0.345], ARRAY[6], 0.345),
  (1, 'waiting', '{"file_path": "/tmp/report.csv"}', 0, NULL, NULL, NULL, NULL);

-- Confirm the seeded data
SELECT 'Agents: ' || COUNT(*) FROM agents;
SELECT 'Steps: ' || COUNT(*) FROM steps;
SELECT 'Signals: ' || COUNT(*) FROM signals;
SELECT 'Runtime Sessions: ' || COUNT(*) FROM runtime_sessions;

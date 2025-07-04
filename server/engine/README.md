# Portico Engine

A Rust-based engine for executing multi-step workflows defined as Agents. The engine can process different step types:
- Python steps (code execution)
- Prompt steps (LLM interactions)
- WebScrape steps (web data extraction)

## Setup

1. Make sure you have Rust and Cargo installed
2. Copy the `.env-example` file to `.env` and update with your settings
3. Build the engine: `cargo build`

```bash
# Development build
cargo build

# Release build
cargo build --release
```

## Running Examples

The engine works with the Portico database to process Agents defined as sequences of Steps.
To run the examples:

1. Make sure your database is set up and running (typically with Supabase)
2. Seed the database with example data: `psql postgresql://postgres:postgres@localhost:54322/postgres -f ../seed.sql`
3. Run the Hacker News example: `psql postgresql://postgres:postgres@localhost:54322/postgres -f ../examples/test_hacker_news.sql`
4. Start the engine: `cargo run`

## Step Types

### Python Steps

Python steps are executed in a Python runtime embedded in the engine. Each Python step:
- Receives a `source` parameter containing the output from the previous step
- Must return a dictionary (`result`) to pass to the next step
- Has access to standard Python libraries

Example Python step:
```python
# Get data from previous step
data = source.get("key", default_value)

# Process the data
processed = do_something(data)

# Return result to next step
result = {
    "processed_data": processed,
    "status": "success"
}
```

### Prompt Steps

Prompt steps send text to an LLM and receive generated output. The prompt can include template variables that are replaced with values from the previous step's output.

Example prompt step:
```
Generate a summary of the following data:
{{data}}

Focus on these key points:
1. Most important trends
2. Anomalies
3. Recommendations
```

### WebScrape Steps

WebScrape steps fetch and parse content from specified URLs. The step content should simply be the URL to scrape.

Example webscrape step:
```
https://news.ycombinator.com
```

## Flow Control

Steps are executed in sequence, with each step receiving the output from the previous step. If a step returns an error, the sequence is aborted.

Steps should handle errors gracefully and include a `status` field in the result to indicate success or failure.

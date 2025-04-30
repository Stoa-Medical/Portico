# Portico SQL Examples

This directory contains example SQL scripts for interacting with the Portico database.

## Running the Hacker News Scraper Test

The `test_hacker_news.sql` file creates a new signal to trigger the Hacker News scraper agent. This demonstrates how to programmatically trigger an agent to run.

### Prerequisites

- Ensure the Portico database is running (typically via Supabase)
- The seed data has been loaded into the database (using `seed.sql`)

### Running the Script

Execute the following command from the project root directory:

```bash
psql postgresql://postgres:postgres@localhost:54322/postgres -f server/examples/test_hacker_news.sql
```

### What This Does

1. Creates a new signal with type 'run' for the Hacker News Scraper agent
2. Includes the current date and a custom output path in the initial data
3. Verifies the signal was created by querying the database

### Checking Results

After running the script, you can check:

1. The newly created signal in the `signals` table
2. Any runtime sessions created in response to the signal
3. The output file at `/tmp/hacker_news_custom.json` (once the agent completes)

### Troubleshooting

If the agent doesn't run as expected, check:

- The agent ID in the script matches the actual ID in your database
- The Portico engine service is running and processing signals
- Database connection parameters are correct

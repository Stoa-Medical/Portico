#!/usr/bin/env python
"""
Test client for the Portico Bridge Service
This script simulates sending different types of signals to the bridge service
through Supabase.

Usage:
    python -m src.test_client run     # Send a run signal
    python -m src.test_client sync    # Send a sync signal
    python -m src.test_client fyi     # Send a FYI signal
"""

import os
import sys
import json
import uuid
import asyncio
from datetime import datetime
from dotenv import load_dotenv
from supabase import create_async_client


async def insert_test_signal(client, signal_type, payload=None):
    """Insert a test signal into Supabase"""

    if payload is None:
        payload = {}

    # Create a unique UUID for the signal
    signal_uuid = str(uuid.uuid4())
    user_uuid = str(uuid.uuid4())

    # Prepare the signal data
    signal_data = {
        "global_uuid": signal_uuid,
        "user_requested_uuid": user_uuid,
        "signal_type": signal_type,
        "rts_id": None,  # Set to null initially, will be updated by the engine
        "initial_data": payload,
    }

    # Insert the signal
    response = await client.table("signals").insert(signal_data).execute()

    if len(response.data) > 0:
        print(f"Successfully inserted {signal_type} signal with UUID: {signal_uuid}")
        print(f"Data: {json.dumps(payload, indent=2)}")
        return signal_uuid
    else:
        print(f"Failed to insert signal: {response.error}")
        return None


async def create_run_signal(client):
    """Create a RUN signal"""

    # Example: Create a new agent
    run_payload = {
        "operation": "CREATE",
        "entity_type": "AGENT",
        "entity_uuid": str(uuid.uuid4()),
        "data": {
            "name": f"Test Agent {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            "description": "Test agent created from test client",
            "agent_state": "stable",
            "agent_name": "TestAgent",
            "agent_type": "python",
        },
    }

    return await insert_test_signal(client, "RUN", run_payload)


async def create_sync_signal(client):
    """Create a SYNC signal"""

    # Example: Sync all agents
    sync_payload = {"scope": "ALL", "entity_types": ["AGENT"]}

    return await insert_test_signal(client, "SYNC", sync_payload)


async def create_fyi_signal(client):
    """Create an FYI signal"""

    # Example: Send an FYI message
    fyi_payload = {
        "message": "This is a test FYI signal",
        "timestamp": datetime.now().isoformat(),
        "metadata": {"source": "test_client.py"},
    }

    return await insert_test_signal(client, "FYI", fyi_payload)


async def main():
    """Main function"""

    if not load_dotenv():
        print("Failed to load .env file. Make sure it exists in the project root.")
        return 1

    # Get Supabase credentials from environment
    supabase_url = os.getenv("SUPABASE_URL")
    supabase_key = os.getenv("SUPABASE_KEY")

    if not supabase_url or not supabase_key:
        print("SUPABASE_URL and SUPABASE_KEY must be set in environment")
        return 1

    # Initialize Supabase client
    client = await create_async_client(supabase_url, supabase_key)

    # Get the signal type from command line
    if len(sys.argv) < 2:
        print("Please specify a signal type: run, sync, or fyi")
        return 1

    signal_type = sys.argv[1].lower()

    try:
        if signal_type == "run":
            await create_run_signal(client)
        elif signal_type == "sync":
            await create_sync_signal(client)
        elif signal_type == "fyi":
            await create_fyi_signal(client)
        else:
            print(f"Unknown signal type: {signal_type}")
            print("Please specify one of: run, sync, or fyi")
            return 1
    except Exception as e:
        print(f"Error: {e}")
        return 1

    return 0


if __name__ == "__main__":
    exit_code = asyncio.run(main())
    sys.exit(exit_code)

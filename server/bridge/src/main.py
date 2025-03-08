"""
This is a Python microservice that:
1. Listens to Supabase Realtime using the Supabase SDK
    - As-of this writing, Rust doesn't have a nice SDK for Supabase realtime.
      Also, keeping this service separate can help keep the code cleaner (separation of concerns)
2. Feeds specific data to the `engine` Rust service using a persistent TCP/IP connection
3. Updates the Signal row accordingly based on runtime result

So the flow is:
                        Triggers:
                        • Signals with "pending" status
                        • Changes to Agents/Steps
┌────────┐         ┌──────────┐         ┌────────┐         ┌────────┐
│  User  │────────▶│`supabase`│────────▶│`bridge`│────────▶│`engine`│
└────────┘         └──────────┘         └────────┘         └────────┘
                        ▲                                      │
                        └──────────────────────────────────────┘
                                Returns:
                                • Updates RuntimeSession
                                • Changes Signal status

"""

import os
import asyncio

from signal import SIGINT, SIGTERM

from supabase import create_async_client
from dotenv import load_dotenv

from src.lib import logger
from src.lib import connect_to_engine, send_signal_to_engine, handle_new_signal


async def shutdown(channel, stop_event):
    """Handle graceful shutdown"""
    logger.info("Shutting down...")
    await channel.unsubscribe()
    stop_event.set()


async def main():
    # Load environment variables
    if not load_dotenv():
        raise RuntimeError(
            "Failed to load `.env` file -- please check if it's at `bridge/.env`!"
        )

    # Get configuration from environment
    supabase_url = os.getenv("SUPABASE_URL")
    supabase_key = os.getenv("SUPABASE_KEY")
    engine_host = os.getenv("ENGINE_CONTAINER_NAME", "engine")
    engine_port = int(os.getenv("ENGINE_PORT", "8888"))

    if not supabase_url or not supabase_key:
        logger.error("SUPABASE_URL and SUPABASE_KEY must be set in environment")
        return

    # Initialize Supabase client
    client = await create_async_client(supabase_url, supabase_key)
    logger.info(f"Initialized Supabase client to {supabase_url}")

    # Send initial DB sync signal to engine
    engine_socket_conn = await connect_to_engine(engine_host, engine_port)
    await send_signal_to_engine(engine_socket_conn, {"server-init": True})

    # Set up Supabase realtime subscriptions
    channel = client.channel("db-changes")

    # Subscribe to changes on the `signals` table (only when added)
    channel.on_postgres_changes(
        event="INSERT",
        table="signals",
        schema="public",
        callback=lambda payload, handler=handle_new_signal: asyncio.create_task(
            handler(payload, engine_socket_conn)
        ),
    )

    # TODO: What's the right way to also listen to updates?
    await channel.subscribe()
    logger.info("Subscribed to Supabase realtime channels")

    # Use asyncio.Event for cleaner termination
    stop_event = asyncio.Event()

    # Set up signal handlers for graceful shutdown
    for sig in (SIGINT, SIGTERM):
        asyncio.get_event_loop().add_signal_handler(
            sig, lambda: asyncio.create_task(shutdown(channel, stop_event))
        )

    logger.info("Portico bridge service started. Press Ctrl+C to exit.")
    await stop_event.wait()
    logger.info("Portico bridge service stopped")


if __name__ == "__main__":
    asyncio.run(main())

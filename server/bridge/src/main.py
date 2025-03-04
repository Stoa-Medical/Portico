"""
This is a Python microservice that:
1. Listens to Supabase Realtime using the Supabase SDK
    - As-of this writing, Rust doesn't have a nice SDK for Supabase realtime.
      Also, keeping this service separate can help keep the code cleaner (separation of concerns)
2. Feeds specific data to the `engine` Rust service using a persistent TCP/IP connection
3. Updates the Signal row accordingly based on runtime result

So the flow is:
  (the User) --> `supabase` <--> `bridge` <--> `engine`
"""

import os
import asyncio

from signal import SIGINT, SIGTERM

from supabase import create_async_client
from dotenv import load_dotenv

from src.lib import logger
from src.lib import setup_realtime, send_signal_to_engine


async def shutdown(channel, stop_event):
    """Handle graceful shutdown"""
    logger.info("Shutting down...")
    await channel.unsubscribe()
    stop_event.set()


async def main():
    # Load environment variables
    if not load_dotenv():
        raise RuntimeError("Failed to load `.env` file -- please check!")

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

    # Set up Supabase realtime subscriptions
    channel = await setup_realtime(client, engine_host, engine_port)

    # Send initial DB sync signal to engine
    await send_signal_to_engine(engine_host, engine_port, {"server-init": True})

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

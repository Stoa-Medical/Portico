"""
This is a Python microservice that:
1. Listens to Supabase Realtime using the Supabase SDK
    - As-of this writing, Rust doesn't have a nice SDK for Supabase realtime.
      Also, keeping this service separate can help keep the code cleaner (separation of concerns)
2. Feeds specific data to the `engine` Rust service using gRPC
3. Updates the Signal row accordingly based on runtime result

So the flow is:
                        Triggers:
                        • Signals (on CREATE). Either `Command` or `Sync`
                        - `Command` requests Create/Update/Delete/Run of specific Agent/Step
                        - `Sync` requests a re-read/serialization of specific Agent/Step (or of all)
┌────────┐         ┌──────────┐         ┌────────┐         ┌────────┐
│  User  │────────▶│`supabase`│────────▶│`bridge`│────────▶│`engine`│
└────────┘         └──────────┘         └────────┘         └────────┘
                        ▲                                      │
                        └──────────────────────────────────────┘
                                Returns:
                                • Writes RuntimeSession
                                • Changes Signal with runtime data (if applicable)

"""

import os
import asyncio

from signal import SIGINT, SIGTERM

from supabase import create_async_client
from dotenv import load_dotenv

from src.lib import logger
from src.lib import BridgeClient, handle_new_signal

# Ensure proto files are generated
from src.proto import build_proto


async def shutdown(channel_list, stop_event, grpc_client):
    """Handle graceful shutdown"""
    logger.info("Shutting down...")
    for channel in channel_list:
        await channel.unsubscribe()

    # Close gRPC connection
    if grpc_client:
        await grpc_client.close()

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
    engine_port = int(os.getenv("ENGINE_PORT", "50051"))

    if not supabase_url or not supabase_key:
        logger.error("SUPABASE_URL and SUPABASE_KEY must be set in environment")
        return

    # Initialize Supabase client
    client = await create_async_client(supabase_url, supabase_key)
    logger.info(f"Initialized Supabase client to {supabase_url}")

    # Generate proto files if they don't exist
    try:
        # Only regenerate if the package was not imported properly
        if not hasattr(build_proto, "generate_proto_code"):
            logger.info("Running build_proto.py to generate gRPC code...")
            build_proto.generate_proto_code()
    except Exception as e:
        logger.error(f"Failed to generate proto files: {e}")
        return

    # Connect to the engine service using gRPC
    grpc_client = BridgeClient(engine_host, engine_port)
    if not await grpc_client.connect():
        logger.error("Failed to connect to engine service via gRPC")
        return

    # Initialize engine with server-init message
    response = await grpc_client.initialize_server()
    if not response or not response.success:
        logger.error("Failed to initialize engine service")
        return

    logger.info(f"Successfully initialized engine service: {response.message}")

    # Set up Supabase realtime subscriptions
    channel_signals = client.channel("signal-inserts")

    # Subscribe to changes on the `signals` table (only when added)
    channel_signals.on_postgres_changes(
        event="INSERT",
        callback=lambda payload, handler=handle_new_signal: asyncio.create_task(
            handler(payload, grpc_client)
        ),
        table="signals",
        schema="public",
    )
    channel_signals.subscribe()

    logger.info("Subscribed to Supabase realtime channels")

    # Use asyncio.Event for cleaner termination
    stop_event = asyncio.Event()

    # Set up signal handlers for graceful shutdown
    for sig in (SIGINT, SIGTERM):
        asyncio.get_event_loop().add_signal_handler(
            sig,
            lambda: asyncio.create_task(
                shutdown([channel_signals], stop_event, grpc_client)
            ),
        )

    logger.info("Portico bridge service started. Press Ctrl+C to exit.")
    await stop_event.wait()
    logger.info("Portico bridge service stopped")


if __name__ == "__main__":
    asyncio.run(main())

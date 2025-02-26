# TODO: This service will:
# 1. Poll Supabase Realtime for `Signals` using SDK
# 2. Feed data to `engine` service using TCP/IP connection
# 3. Update the `Signals` based on the `engine` results

import os
import socket
import asyncio
from signal import SIGINT, SIGTERM
from supabase import create_client
from dotenv import load_dotenv


async def handle_signal_changes(payload):
    print("Change received:", payload)
    # Send over TCP/IP connection to `engine` service
    ...


async def main():
    # Load dotenv
    load_dotenv()

    # # Open TCP connection to `engine` service
    # engine_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    # engine_socket.connect((os.getenv("ENGINE_CONTAINER_NAME"), 8888))

    supabase_client = create_client(
        os.getenv("SUPABASE_URL"),
        os.getenv("SUPABASE_KEY"),
    )

    # Set up realtime subscription
    channel = supabase_client.channel("db-changes")

    # Subscribe to changes on the signals table
    channel.on(
        "postgres_changes",
        event="*",  # or specific events like 'INSERT', 'UPDATE', 'DELETE'
        schema="public",
        table="signals",
        callback=handle_signal_changes,
    )

    # Start the subscription
    await channel.subscribe()

    # Use asyncio.Event for cleaner termination
    stop_event = asyncio.Event()

    # Set up signal handlers for graceful shutdown
    for sig in (SIGINT, SIGTERM):
        asyncio.get_event_loop().add_signal_handler(
            sig, lambda: asyncio.create_task(shutdown(channel, stop_event))
        )

    print("Listening for changes on signals table. Press Ctrl+C to exit.")
    await stop_event.wait()
    print("Exited cleanly")


async def shutdown(channel, stop_event):
    print("Shutting down...")
    await channel.unsubscribe()
    stop_event.set()


if __name__ == "__main__":
    asyncio.run(main())

import asyncio
import json
import socket
import logging

from typing import Any

from supabase import AsyncClient
from realtime import AsyncRealtimeChannel

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("portico-bridge")


# Supabase setup
async def setup_realtime(
    client: AsyncClient, sock: socket.socket
) -> AsyncRealtimeChannel:
    """Set up realtime subscriptions for Supabase tables"""
    channel = client.channel("db-changes")

    # Subscribe to changes on the `signals` table (only when added)
    channel.on_postgres_changes(
        event="INSERT",
        table="signals",
        schema="public",
        callback=lambda payload, handler=_handle_new_signal: asyncio.create_task(
            handler(payload, sock)
        ),
    )

    # TODO: What's the right way to also listen to updates?

    await channel.subscribe()
    logger.info("Subscribed to Supabase realtime channels")
    return channel


# Engine communication functions
async def send_signal_to_engine(
    sock: socket.socket, data: dict[str, Any]
) -> dict[str, Any] | None:
    """Send a signal to the engine and get the response"""
    try:
        # Prepare the signal message
        message = json.dumps(data).encode("utf-8")

        # Send the message
        sock.sendall(message)
        logger.info(f"Sent to engine: {data}")

        # Receive the response
        response_data = sock.recv(4096)
        if not response_data:
            logger.error("Received empty response from engine")
            return None

        # Parse the response
        response = json.loads(response_data.decode("utf-8"))
        logger.info(f"Received response from engine: {response}")
        return response
    except Exception as e:
        logger.error(f"Error communicating with engine: {e}")
        return None


async def connect_to_engine(host: str, port: int) -> socket.socket | None:
    """Establish connection to the engine service"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((host, port))
        logger.info(f"Connected to engine at {host}:{port}")
        return sock
    except Exception as e:
        logger.error(f"Failed to connect to engine: {e}")
        return None


# === "Private" functions (for organization sake) ===


async def _handle_new_signal(payload: dict[str, Any], sock: socket.socket) -> None:
    """Handle a new signal inserted into the signals table"""
    logger.info(f"🔔 New signal detected: {payload}")

    try:
        # Send signal to engine
        await send_signal_to_engine(
            sock,
            payload,
        )
        logger.info("Sent payload")
    except Exception as e:
        logger.error(f"Error handling new signal: {e}")


async def _handle_update(payload: dict[str, Any], sock: socket.socket) -> None:
    """Handle changes to agents"""
    logger.info(f"🔃 Change detected: {payload}")

    # Notify the engine to sync its agent data
    await send_signal_to_engine(sock, payload)
    logger.info("DB sync request sent to engine")

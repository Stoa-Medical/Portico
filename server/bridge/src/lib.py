import json
import socket
import logging

from typing import Any

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("portico-bridge")


# Engine communication functions
async def send_signal_to_engine(
    sock: socket.socket, data: dict[str, Any], meta: str = "🏹"
) -> dict[str, Any] | None:
    """Send a signal to the engine and get the response"""
    try:
        # Prepare the signal message
        message = json.dumps(data).encode("utf-8")

        # Prefix with message length (4 bytes, big-endian)
        message_length = len(message)
        length_prefix = message_length.to_bytes(4, byteorder="big")

        # Send the length prefix followed by the message
        sock.sendall(length_prefix + message)
        logger.info(f"Sent message to engine ({meta})")

        # Receive the response - first read 4-byte length
        length_data = sock.recv(4)
        if not length_data or len(length_data) < 4:
            logger.error("Failed to receive message length from engine")
            return None

        response_length = int.from_bytes(length_data, byteorder="big")

        # Now read the exact message length
        response_data = sock.recv(response_length)
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


async def handle_new_signal(payload: dict[str, Any], sock: socket.socket) -> None:
    """Handle a new signal inserted into the signals table"""
    logger.info(f"🔔 New signal detected: {payload}")

    try:
        # Send signal to engine
        await send_signal_to_engine(sock, payload, "handle_new_signal")
    except Exception as e:
        logger.error(f"Error handling new signal: {e}")


async def handle_general_update(payload: dict[str, Any], sock: socket.socket) -> None:
    """Handle changes to agents"""
    data = payload.get("data", {})
    logger.info(
        f"🔃 General Update detected: {payload.get('ids')} | {data.get('table')} | {data.get('type')}"
    )

    # Notify the engine to sync its agent data
    await send_signal_to_engine(sock, payload, "handle_general_update")

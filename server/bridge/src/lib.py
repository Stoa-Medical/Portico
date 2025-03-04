import asyncio
import json
import socket
import logging

from enum import Enum
from typing import Any

from supabase import AsyncClient
from realtime import AsyncRealtimeChannel


# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("portico-bridge")


# Signal types that the bridge can send to the engine
class SignalType(Enum):
    AGENT_RUN = "AGENT_RUN"
    AGENT_STOP = "AGENT_STOP"
    DB_SYNC = "DB_SYNC"
    
    def __str__(self):
        return self.value

# Supabase setup
async def setup_realtime(
    client: AsyncClient, 
    engine_host: str, 
    engine_port: int
) -> AsyncRealtimeChannel:
    """Set up realtime subscriptions for Supabase tables"""
    channel = client.channel("db-changes")
    
    # Subscribe to changes on the signals table
    channel.on(
        "postgres_changes",
        event="INSERT",
        schema="public",
        table="signals",
        callback=lambda payload: asyncio.create_task(
            _handle_new_singal(payload, client, engine_host, engine_port)
        )
    )
    
    # Subscribe to updates on the signals table
    channel.on(
        "postgres_changes",
        event="UPDATE",
        schema="public",
        table="signals",
        callback=lambda payload: asyncio.create_task(
            _handle_signal_update(payload, engine_host, engine_port)
        )
    )
    
    # Subscribe to changes on the agents table
    channel.on(
        "postgres_changes",
        event="*",
        schema="public",
        table="agents",
        callback=lambda payload: asyncio.create_task(
            _handle_agent_change(payload, engine_host, engine_port)
        )
    )
    
    await channel.subscribe()
    logger.info("Subscribed to Supabase realtime channels")
    return channel

# Engine communication functions
async def send_signal_to_engine(
    host: str, 
    port: int, 
    signal_type: SignalType, 
    data: dict[str, Any]
) -> dict[str, Any] | None:
    """Send a signal to the engine and get the response"""
    sock = await _connect_to_engine(host, port)
    if not sock:
        return None
    
    try:
        # Prepare the signal message
        message = json.dumps({
            "signal_type": str(signal_type),
            "data": data
        }).encode('utf-8')
        
        # Send the message
        sock.sendall(message)
        logger.info(f"Sent {signal_type} signal to engine")
        
        # Receive the response
        response_data = sock.recv(4096)
        if not response_data:
            logger.error("Received empty response from engine")
            return None
        
        # Parse the response
        response = json.loads(response_data.decode('utf-8'))
        logger.info(f"Received response from engine: {response}")
        return response
    except Exception as e:
        logger.error(f"Error communicating with engine: {e}")
        return None
    finally:
        # Close the socket
        if sock:
            sock.close()


# === "Private" functions (for organization sake) ===

async def _connect_to_engine(host: str, port: int) -> socket.socket | None:
    """Establish connection to the engine service"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((host, port))
        logger.info(f"Connected to engine at {host}:{port}")
        return sock
    except Exception as e:
        logger.error(f"Failed to connect to engine: {e}")
        return None


async def _handle_new_singal(
    payload: dict[str, Any], 
    client: AsyncClient,
    engine_host: str,
    engine_port: int
) -> None:
    """Handle a new signal inserted into the signals table"""
    logger.info(f"New signal detected: {payload}")
    
    try:
        # Extract signal data
        signal_data = payload.get("new", {})
        signal_id = signal_data.get("id")
        agent_id = signal_data.get("agent_id")
        
        if not signal_id or not agent_id:
            logger.error(f"Invalid signal data: {signal_data}")
            return
        
        # Update signal status to "processing"
        await _update_signal(client, signal_id, {"status": "PROCESSING"})
        
        # Send signal to engine
        response = await send_signal_to_engine(
            engine_host,
            engine_port,
            SignalType.AGENT_RUN,
            {
                "signal_id": str(signal_id),
                "agent_id": str(agent_id),
                "starting_data": signal_data.get("initial_data", {})
            }
        )
        
        if response and response.get("status") == "success":
            # Store the session ID for later reference
            session_id = response.get("session_id")
            await _update_signal(client, signal_id, {"session_id": session_id})
            logger.info(f"Signal {signal_id} processing started with session {session_id}")
        else:
            # Update signal status to "failed"
            error_msg = response.get("message") if response else "Failed to communicate with engine"
            await _update_signal(client, signal_id, {
                "status": "FAILED",
                "error_message": error_msg
            })
            logger.error(f"Failed to process signal {signal_id}: {error_msg}")
    
    except Exception as e:
        logger.error(f"Error handling new signal: {e}")


async def _handle_signal_update(
    payload: dict[str, Any], 
    engine_host: str,
    engine_port: int
) -> None:
    """Handle updates to existing signals"""
    logger.info(f"Signal update detected: {payload}")
    
    try:
        # Extract signal data
        old_data = payload.get("old", {})
        new_data = payload.get("new", {})
        signal_id = new_data.get("id")
        
        # Check if this is a cancellation request
        if old_data.get("status") != "CANCELLED" and new_data.get("status") == "CANCELLED":
            session_id = new_data.get("session_id")
            if session_id:
                # Send cancellation signal to engine
                await send_signal_to_engine(
                    engine_host,
                    engine_port,
                    SignalType.AGENT_STOP,
                    {
                        "signal_id": str(signal_id),
                        "session_id": session_id
                    }
                )
                logger.info(f"Cancellation request sent for signal {signal_id}")
    
    except Exception as e:
        logger.error(f"Error handling signal update: {e}")

async def _handle_agent_change(
    payload: dict[str, Any], 
    engine_host: str,
    engine_port: int
) -> None:
    """Handle changes to agents"""
    logger.info(f"Agent change detected: {payload}")
    
    # Notify the engine to sync its agent data
    await send_signal_to_engine(
        engine_host,
        engine_port,
        SignalType.DB_SYNC,
        {"table": "agents"}
    )
    logger.info("DB sync request sent to engine")

async def _update_signal(client: AsyncClient, signal_id: str, data: dict[str, Any]) -> bool:
    """Update a signal in Supabase with the provided data"""
    try:
        await client.table("signals").update(data).eq("id", signal_id).execute()
        logger.info(f"Updated signal {signal_id} with data: {data}")
        return True
    except Exception as e:
        logger.error(f"Error updating signal {signal_id}: {e}")
        return False

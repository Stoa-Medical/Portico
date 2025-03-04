# TODO: This service will:
# 1. Poll Supabase Realtime for `Signals` using SDK
# 2. Feed data to `engine` service using TCP/IP connection
# 3. Update the `Signals` based on the `engine` results

import os
import json
import socket
import asyncio
import logging
from signal import SIGINT, SIGTERM
from typing import Dict, Any, Optional
from enum import Enum
from supabase import create_async_client, AsyncClient
from dotenv import load_dotenv

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

# Engine communication functions
async def connect_to_engine(host: str, port: int) -> Optional[socket.socket]:
    """Establish connection to the engine service"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((host, port))
        logger.info(f"Connected to engine at {host}:{port}")
        return sock
    except Exception as e:
        logger.error(f"Failed to connect to engine: {e}")
        return None

async def send_signal_to_engine(
    host: str, 
    port: int, 
    signal_type: SignalType, 
    data: Dict[str, Any]
) -> Optional[Dict[str, Any]]:
    """Send a signal to the engine and get the response"""
    sock = await connect_to_engine(host, port)
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

# Supabase functions
async def update_signal(client: AsyncClient, signal_id: str, data: Dict[str, Any]):
    """Update a signal in Supabase with the provided data"""
    try:
        await client.table("signals").update(data).eq("id", signal_id).execute()
        logger.info(f"Updated signal {signal_id} with data: {data}")
        return True
    except Exception as e:
        logger.error(f"Error updating signal {signal_id}: {e}")
        return False

async def update_signal_status(client: AsyncClient, signal_id: str, status: str):
    """Update the status of a signal in Supabase"""
    return await update_signal(client, signal_id, {"status": status})

async def update_signal_session(client: AsyncClient, signal_id: str, session_id: str):
    """Update the session_id of a signal in Supabase"""
    return await update_signal(client, signal_id, {"session_id": session_id})

async def update_signal_error(client: AsyncClient, signal_id: str, error_message: str):
    """Update a signal with error information"""
    return await update_signal(client, signal_id, {
        "status": "FAILED",
        "error_message": error_message
    })

# Signal handlers
async def handle_new_signal(
    payload: Dict[str, Any], 
    client: AsyncClient,
    engine_host: str,
    engine_port: int
):
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
        await update_signal_status(client, signal_id, "PROCESSING")
        
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
            await update_signal_session(client, signal_id, session_id)
            logger.info(f"Signal {signal_id} processing started with session {session_id}")
        else:
            # Update signal status to "failed"
            error_msg = response.get("message") if response else "Failed to communicate with engine"
            await update_signal_error(client, signal_id, error_msg)
            logger.error(f"Failed to process signal {signal_id}: {error_msg}")
    
    except Exception as e:
        logger.error(f"Error handling new signal: {e}")

async def handle_signal_update(
    payload: Dict[str, Any], 
    client: AsyncClient,
    engine_host: str,
    engine_port: int
):
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

async def handle_agent_change(
    payload: Dict[str, Any], 
    client: AsyncClient,
    engine_host: str,
    engine_port: int
):
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

async def setup_realtime(
    client: AsyncClient, 
    engine_host: str, 
    engine_port: int
):
    """Set up realtime subscriptions for Supabase tables"""
    channel = client.channel("db-changes")
    
    # Subscribe to changes on the signals table
    channel.on(
        "postgres_changes",
        event="INSERT",
        schema="public",
        table="signals",
        callback=lambda payload: asyncio.create_task(
            handle_new_signal(payload, client, engine_host, engine_port)
        )
    )
    
    # Subscribe to updates on the signals table
    channel.on(
        "postgres_changes",
        event="UPDATE",
        schema="public",
        table="signals",
        callback=lambda payload: asyncio.create_task(
            handle_signal_update(payload, client, engine_host, engine_port)
        )
    )
    
    # Subscribe to changes on the agents table
    channel.on(
        "postgres_changes",
        event="*",
        schema="public",
        table="agents",
        callback=lambda payload: asyncio.create_task(
            handle_agent_change(payload, client, engine_host, engine_port)
        )
    )
    
    await channel.subscribe()
    logger.info("Subscribed to Supabase realtime channels")
    return channel

async def shutdown(channel, stop_event):
    """Handle graceful shutdown"""
    logger.info("Shutting down...")
    await channel.unsubscribe()
    stop_event.set()

async def main():
    # Load environment variables
    load_dotenv()
    
    # Get configuration from environment
    supabase_url = os.getenv("SUPABASE_URL")
    supabase_key = os.getenv("SUPABASE_KEY")
    engine_host = os.getenv("ENGINE_CONTAINER_NAME", "engine")
    engine_port = int(os.getenv("ENGINE_PORT", "8888"))
    
    if not supabase_url or not supabase_key:
        logger.error("SUPABASE_URL and SUPABASE_KEY must be set in environment")
        return
    
    # Initialize Supabase client
    client = create_async_client(supabase_url, supabase_key)
    logger.info(f"Initialized Supabase client to {supabase_url}")
    
    # Set up Supabase realtime subscriptions
    channel = await setup_realtime(client, engine_host, engine_port)
    
    # Send initial DB sync signal to engine
    await send_signal_to_engine(
        engine_host, 
        engine_port, 
        SignalType.DB_SYNC, 
        {"initial": True}
    )
    
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

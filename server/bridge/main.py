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
from supabase import create_client, Client
from dotenv import load_dotenv

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("portico-bridge")

# Signal types that the bridge can send to the engine
class SignalType:
    AGENT_RUN = "AGENT_RUN"
    AGENT_STOP = "AGENT_STOP"
    DB_SYNC = "DB_SYNC"

class EngineCommunicator:
    """Handles communication with the Rust engine service"""
    
    def __init__(self, host: str, port: int):
        self.host = host
        self.port = port
        self.socket = None
        logger.info(f"Initialized engine communicator to {host}:{port}")
    
    async def connect(self) -> bool:
        """Establish connection to the engine service"""
        try:
            # Create a new socket for each connection to avoid stale connections
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.connect((self.host, self.port))
            logger.info(f"Connected to engine at {self.host}:{self.port}")
            return True
        except Exception as e:
            logger.error(f"Failed to connect to engine: {e}")
            return False
    
    async def send_signal(self, signal_type: str, data: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Send a signal to the engine and get the response"""
        if not await self.connect():
            return None
        
        try:
            # Prepare the signal message
            message = json.dumps({
                "signal_type": signal_type,
                "data": data
            }).encode('utf-8')
            
            # Send the message
            self.socket.sendall(message)
            logger.info(f"Sent {signal_type} signal to engine")
            
            # Receive the response
            response_data = self.socket.recv(4096)
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
            if self.socket:
                self.socket.close()
                self.socket = None

class SupabaseHandler:
    """Handles interactions with Supabase"""
    
    def __init__(self, url: str, key: str):
        self.client = create_client(url, key)
        logger.info(f"Initialized Supabase client to {url}")
    
    async def setup_realtime(self, engine_comm: EngineCommunicator):
        """Set up realtime subscriptions for Supabase tables"""
        channel = self.client.channel("db-changes")
        
        # Subscribe to changes on the signals table
        channel.on(
            "postgres_changes",
            event="INSERT",
            schema="public",
            table="signals",
            callback=lambda payload: asyncio.create_task(self.handle_new_signal(payload, engine_comm))
        )
        
        # Subscribe to updates on the signals table
        channel.on(
            "postgres_changes",
            event="UPDATE",
            schema="public",
            table="signals",
            callback=lambda payload: asyncio.create_task(self.handle_signal_update(payload, engine_comm))
        )
        
        # Subscribe to changes on the agents table
        channel.on(
            "postgres_changes",
            event="*",
            schema="public",
            table="agents",
            callback=lambda payload: asyncio.create_task(self.handle_agent_change(payload, engine_comm))
        )
        
        await channel.subscribe()
        logger.info("Subscribed to Supabase realtime channels")
        return channel
    
    async def handle_new_signal(self, payload: Dict[str, Any], engine_comm: EngineCommunicator):
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
            await self.update_signal_status(signal_id, "PROCESSING")
            
            # Send signal to engine
            response = await engine_comm.send_signal(
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
                await self.update_signal_session(signal_id, session_id)
                logger.info(f"Signal {signal_id} processing started with session {session_id}")
            else:
                # Update signal status to "failed"
                error_msg = response.get("message") if response else "Failed to communicate with engine"
                await self.update_signal_error(signal_id, error_msg)
                logger.error(f"Failed to process signal {signal_id}: {error_msg}")
        
        except Exception as e:
            logger.error(f"Error handling new signal: {e}")
    
    async def handle_signal_update(self, payload: Dict[str, Any], engine_comm: EngineCommunicator):
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
                    await engine_comm.send_signal(
                        SignalType.AGENT_STOP,
                        {
                            "signal_id": str(signal_id),
                            "session_id": session_id
                        }
                    )
                    logger.info(f"Cancellation request sent for signal {signal_id}")
        
        except Exception as e:
            logger.error(f"Error handling signal update: {e}")
    
    async def handle_agent_change(self, payload: Dict[str, Any], engine_comm: EngineCommunicator):
        """Handle changes to agents"""
        logger.info(f"Agent change detected: {payload}")
        
        # Notify the engine to sync its agent data
        await engine_comm.send_signal(
            SignalType.DB_SYNC,
            {"table": "agents"}
        )
        logger.info("DB sync request sent to engine")
    
    async def update_signal_status(self, signal_id: str, status: str):
        """Update the status of a signal in Supabase"""
        try:
            await self.client.table("signals").update({"status": status}).eq("id", signal_id).execute()
            logger.info(f"Updated signal {signal_id} status to {status}")
        except Exception as e:
            logger.error(f"Error updating signal status: {e}")
    
    async def update_signal_session(self, signal_id: str, session_id: str):
        """Update the session_id of a signal in Supabase"""
        try:
            await self.client.table("signals").update({"session_id": session_id}).eq("id", signal_id).execute()
            logger.info(f"Updated signal {signal_id} with session_id {session_id}")
        except Exception as e:
            logger.error(f"Error updating signal session: {e}")
    
    async def update_signal_error(self, signal_id: str, error_message: str):
        """Update a signal with error information"""
        try:
            await self.client.table("signals").update({
                "status": "FAILED",
                "error_message": error_message
            }).eq("id", signal_id).execute()
            logger.info(f"Updated signal {signal_id} with error: {error_message}")
        except Exception as e:
            logger.error(f"Error updating signal error: {e}")

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
    
    # Initialize components
    engine_comm = EngineCommunicator(engine_host, engine_port)
    supabase_handler = SupabaseHandler(supabase_url, supabase_key)
    
    # Set up Supabase realtime subscriptions
    channel = await supabase_handler.setup_realtime(engine_comm)
    
    # Send initial DB sync signal to engine
    await engine_comm.send_signal(SignalType.DB_SYNC, {"initial": True})
    
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

async def shutdown(channel, stop_event):
    """Handle graceful shutdown"""
    logger.info("Shutting down...")
    await channel.unsubscribe()
    stop_event.set()

if __name__ == "__main__":
    asyncio.run(main())

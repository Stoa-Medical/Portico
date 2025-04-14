import logging
import grpc
from typing import Any, Dict, Optional
from google.protobuf.struct_pb2 import Struct
from google.protobuf.json_format import ParseDict

# Import the generated gRPC code
# Note: run build_proto.py first to generate these modules
from src.proto import bridge_message_pb2 as pb2
from src.proto import bridge_message_pb2_grpc as pb2_grpc

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("portico-bridge")


# Helper function to convert Python dict to Protobuf Struct
def dict_to_struct(data: Dict[str, Any]) -> Struct:
    """Convert a Python dictionary to a Protobuf Struct"""
    struct = Struct()
    ParseDict(data, struct)
    return struct


# gRPC client class
class BridgeClient:
    """gRPC client for communicating with the engine service"""

    def __init__(self, host: str, port: int):
        self.host = host
        self.port = port
        self.channel = None
        self.stub = None

    async def connect(self) -> bool:
        """Connect to the gRPC server"""
        try:
            # Create an insecure channel
            address = f"{self.host}:{self.port}"
            self.channel = grpc.aio.insecure_channel(address)
            self.stub = pb2_grpc.BridgeServiceStub(self.channel)  # type: ignore
            logger.info(f"Created gRPC channel to {address}")

            # Initialize the server
            response = await self.initialize_server()
            if response and response.success:
                logger.info(
                    f"Successfully initialized gRPC connection: {response.message}"
                )
                return True
            else:
                logger.error("Failed to initialize gRPC connection")
                return False
        except Exception as e:
            logger.error(f"Failed to connect to gRPC server: {e}")
            return False

    async def initialize_server(self) -> Optional[Any]:
        """Initialize the server connection"""
        try:
            request = pb2.ServerInitRequest(server_init=True)  # type: ignore
            response = await self.stub.InitServer(request)  # type: ignore
            return response
        except Exception as e:
            logger.error(f"Error initializing server: {e}")
            return None

    async def send_signal(self, data: Dict[str, Any], meta: str = "🏹") -> bool:
        """Send a signal to the engine"""
        try:
            # Check if 'data' key exists with table and record fields
            if "data" in data and isinstance(data["data"], dict):
                supabase_data = data["data"]
                table = supabase_data.get("table", "")
                event_type = supabase_data.get("type", "")
                record = supabase_data.get("record", {})

                # Convert record to Struct
                record_struct = dict_to_struct(record)

                # Create SupabaseData message
                supabase_proto = pb2.SupabaseData(  # type: ignore
                    table=table, type=event_type, record=record_struct
                )

                if table == "signals":
                    # Create SignalRequest
                    request = pb2.SignalRequest(data=supabase_proto)  # type: ignore
                    response = await self.stub.CreateSignal(request)  # type: ignore
                elif table == "agents":
                    # Create AgentRequest
                    request = pb2.AgentRequest(data=supabase_proto)  # type: ignore

                    if event_type == "INSERT":
                        response = await self.stub.CreateAgent(request)  # type: ignore
                    elif event_type == "UPDATE":
                        response = await self.stub.UpdateAgent(request)  # type: ignore
                    elif event_type == "DELETE":
                        response = await self.stub.DeleteAgent(request)  # type: ignore
                    else:
                        logger.error(f"Unsupported event type: {event_type}")
                        return False
                else:
                    logger.error(f"Unsupported table: {table}")
                    return False

                if response and response.success:
                    logger.info(f"Successfully sent {meta} message: {response.message}")
                    return True
                else:
                    logger.error(f"Failed to send {meta} message to engine")
                    return False
            elif "server-init" in data:
                # This is an initialization message
                return await self.initialize_server() is not None
            else:
                logger.error(f"Invalid message format: {data}")
                return False
        except Exception as e:
            logger.error(f"Error sending message to engine: {e}")
            return False

    async def close(self):
        """Close the gRPC channel"""
        if self.channel:
            await self.channel.close()
            logger.info("Closed gRPC channel")


# Public API for the bridge service
async def handle_new_signal(payload: Dict[str, Any], client: BridgeClient) -> None:
    """Handle a new signal inserted into the signals table"""
    logger.info(f"🔔 New signal detected: {payload}")

    try:
        # Send signal to engine
        await client.send_signal(payload, "handle_new_signal")
    except Exception as e:
        logger.error(f"Error handling new signal: {e}")


async def handle_general_update(payload: Dict[str, Any], client: BridgeClient) -> None:
    """Handle changes to agents"""
    data = payload.get("data", {})
    logger.info(
        f"🔃 General Update detected: {payload.get('ids')} | {data.get('table')} | {data.get('type')}"
    )

    # Notify the engine to sync its agent data
    await client.send_signal(payload, "handle_general_update")

import logging
import json
import uuid
import grpc
from typing import Any, Dict, Optional, cast
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
        self.channel: Optional[grpc.aio.Channel] = None
        self.stub: Optional[pb2_grpc.BridgeServiceStub] = None

    async def connect(self) -> bool:
        """Connect to the gRPC server"""
        try:
            # Create an insecure channel
            address = f"{self.host}:{self.port}"
            self.channel = grpc.aio.insecure_channel(address)
            self.stub = pb2_grpc.BridgeServiceStub(self.channel)
            logger.info(f"Created gRPC channel to {address}")
            return True
        except Exception as e:
            logger.error(f"Failed to connect to gRPC server: {e}")
            return False

    async def initialize_server(self) -> Optional[Any]:
        """Initialize the server connection"""
        try:
            if not self.stub:
                logger.error("gRPC stub not initialized")
                return None
            # Use pb2 namespace to access the generated classes
            request = pb2.ServerInitRequest(server_init=True)
            response = await self.stub.InitServer(request)
            return response
        except Exception as e:
            logger.error(f"Error initializing server: {e}")
            return None

    async def process_signal(self, signal_request: Any) -> Optional[Any]:
        """Send a signal request to the engine"""
        try:
            if not self.stub:
                logger.error("gRPC stub not initialized")
                return None
            response = await self.stub.ProcessSignal(signal_request)
            return response
        except Exception as e:
            logger.error(f"Error processing signal: {sanitize_data(str(e))}")
            return None

    async def send_signal(self, data: Dict[str, Any], meta: str = "signal") -> bool:
        """Send a signal to the engine using the unified SignalRequest structure"""
        try:
            # Handle the server init case separately
            if "server-init" in data:
                response = await self.initialize_server()
                return response is not None and getattr(response, "success", False)

            # Process the actual signal based on the data
            signal_request = await create_signal_request(data)
            if signal_request:
                response = await self.process_signal(signal_request)
                if response and getattr(response, "success", False):
                    logger.info(
                        f"Successfully sent {meta} message: {sanitize_data(getattr(response, 'message', ''))}"
                    )
                    return True
                else:
                    error_msg = (
                        getattr(response, "message", "No response received")
                        if response
                        else "No response received"
                    )
                    logger.error(f"Failed to send {meta} message: {error_msg}")
                    return False
            else:
                logger.error(
                    f"Failed to create signal request from data: {sanitize_data(data)}"
                )
                return False
        except Exception as e:
            logger.error(f"Error sending message to engine: {sanitize_data(str(e))}")
            return False

    async def close(self):
        """Close the gRPC channel"""
        if self.channel:
            await self.channel.close()
            logger.info("Closed gRPC channel")


async def create_signal_request(data: Dict[str, Any]) -> Optional[Any]:
    """Create a SignalRequest from the Supabase payload"""
    try:
        # Extract record data from the Supabase payload
        supabase_data = data.get("data", {})
        record = supabase_data.get("record", {})

        if not record:
            logger.error("No record found in data payload")
            return None

        # Create the SignalRequest
        global_uuid = str(record.get("global_uuid", uuid.uuid4()))
        user_requested_uuid = str(record.get("user_requested_uuid", ""))

        # Determine signal type
        signal_type_str = record.get("signal_type", "").upper()
        if not signal_type_str:
            logger.error("No signal_type found in record")
            return None

        try:
            # Access via the pb2 namespace
            signal_type = pb2.SignalType.Value(signal_type_str)
        except ValueError:
            logger.error(f"Invalid signal_type: {signal_type_str}")
            return None

        # Extract initial_data JSON
        initial_data = record.get("initial_data", {})
        if isinstance(initial_data, str):
            try:
                initial_data = json.loads(initial_data)
            except json.JSONDecodeError:
                logger.error(f"Invalid JSON in initial_data: {initial_data}")
                initial_data = {}

        # Create the base request via pb2 namespace
        request = pb2.SignalRequest(
            global_uuid=global_uuid,
            user_requested_uuid=user_requested_uuid,
            signal_type=signal_type,
        )

        # Handle payload based on signal type - access via pb2 namespace
        if signal_type == pb2.SignalType.COMMAND:
            command_payload = create_command_payload(initial_data)
            request.command.CopyFrom(command_payload)
        elif signal_type == pb2.SignalType.SYNC:
            sync_payload = create_sync_payload(initial_data)
            request.sync.CopyFrom(sync_payload)
        elif signal_type == pb2.SignalType.FYI:
            request.fyi_data.CopyFrom(dict_to_struct(initial_data))

        return request
    except Exception as e:
        logger.error(f"Error creating signal request: {str(e)}")
        return None


def create_command_payload(data: Dict[str, Any]) -> Any:
    """Create a CommandPayload from the initial_data"""
    # Get operation type
    operation_str = data.get("operation", "").upper()
    try:
        # Access via pb2 namespace
        operation = pb2.CommandOperation.Value(operation_str)
    except ValueError:
        logger.error(f"Invalid operation: {operation_str}")
        # Default via pb2 namespace
        operation = pb2.CommandOperation.CREATE

    # Get entity type
    entity_type_str = data.get("entity_type", "").upper()
    try:
        # Access via pb2 namespace
        entity_type = pb2.EntityType.Value(entity_type_str)
    except ValueError:
        logger.error(f"Invalid entity_type: {entity_type_str}")
        # Default via pb2 namespace
        entity_type = pb2.EntityType.AGENT

    # Get entity UUID
    entity_uuid = str(data.get("entity_uuid", ""))

    # Get payload data
    payload_data = data.get("data", {})

    # Get update fields
    update_fields = data.get("update_fields", [])

    # Create and return via pb2 namespace
    return pb2.CommandPayload(
        operation=operation,
        entity_type=entity_type,
        entity_uuid=entity_uuid,
        data=dict_to_struct(payload_data),
        update_fields=update_fields,
    )


def create_sync_payload(data: Dict[str, Any]) -> Any:
    """Create a SyncPayload from the initial_data"""
    # Get sync scope
    scope_str = data.get("scope", "ALL").upper()
    try:
        # Access via pb2 namespace
        scope = pb2.SyncScope.Value(scope_str)
    except ValueError:
        logger.error(f"Invalid scope: {scope_str}")
        # Default via pb2 namespace
        scope = pb2.SyncScope.ALL

    # Get entity UUIDs
    entity_uuids = [str(uuid_val) for uuid_val in data.get("entity_uuids", [])]

    # Get entity types
    entity_types_str = data.get("entity_types", [])
    entity_types = []
    for et_str in entity_types_str:
        try:
            # Access via pb2 namespace
            et = pb2.EntityType.Value(et_str.upper())
            entity_types.append(et)
        except ValueError:
            logger.error(f"Invalid entity_type: {et_str}")

    # Create and return via pb2 namespace
    return pb2.SyncPayload(
        scope=scope, entity_uuids=entity_uuids, entity_types=entity_types
    )


# Sanitize data by removing null characters that Postgres can't handle
def sanitize_data(data: Any) -> Any:
    """Sanitize data by removing null characters that Postgres can't handle."""
    if isinstance(data, str):
        return data.replace("\u0000", "")
    elif isinstance(data, dict):
        return {k: sanitize_data(v) for k, v in data.items()}
    elif isinstance(data, list):
        return [sanitize_data(v) for v in data]
    return data


# Public API for the bridge service
async def handle_new_signal(payload: Dict[str, Any], client: BridgeClient) -> None:
    """Handle a new signal inserted into the signals table"""
    # Sanitize the payload before any processing
    safe_payload = sanitize_data(payload)
    logger.info(f"🔔 New signal detected: {safe_payload}")

    try:
        # Send sanitized signal to engine
        success = await client.send_signal(safe_payload, "signal")
        if not success:
            logger.error("Failed to process signal in engine service")
    except Exception as e:
        logger.error(f"Error handling new signal: {str(e)}")

import logging
import json
import uuid
import grpc
from typing import Any
from google.protobuf.struct_pb2 import Struct
from google.protobuf.json_format import ParseDict
from result import Ok, Err, Result
from pydian import get

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
def dict_to_struct(data: dict[str, Any]) -> Struct:
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
        self.channel: grpc.aio.Channel | None = None
        self.stub: pb2_grpc.BridgeServiceStub | None = None

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

    async def initialize_server(self) -> Any:
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

    async def process_signal(self, signal_request: Any) -> Any:
        """Send a signal request to the engine"""
        try:
            if not self.stub:
                logger.error("gRPC stub not initialized")
                return None
            response = await self.stub.ProcessSignal(signal_request)

            # Log the runtime_session_uuid if present
            if (
                response
                and hasattr(response, "runtime_session_uuid")
                and response.runtime_session_uuid
            ):
                logger.info(
                    f"Received runtime_session_uuid: {response.runtime_session_uuid}"
                )

                # In a real implementation, you might want to update the database with this UUID
                # to link the signal with the runtime session
                # Example: update_signal_with_rts_id(signal_request.signal_id, response.runtime_session_uuid)

            return response
        except Exception as e:
            logger.error(f"Error processing signal: {sanitize_data(str(e))}")
            return None

    async def send_signal(self, data: dict[str, Any], meta: str = "signal") -> bool:
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


async def create_signal_request(data: dict[str, Any]) -> Any:
    """Create a SignalRequest from the Supabase payload"""
    try:
        # Extract record data from the Supabase payload using pydian get
        record = get(data, "data.record", {})

        if not record:
            logger.error("No record found in data payload")
            return None

        # Extract the necessary fields from the record
        # Get the signal ID directly from the record
        signal_id = get(record, "id", 0)

        # Get the agent ID directly from the record
        agent_id = get(record, "agent_id", 0)

        if agent_id:
            logger.info(f"Processing signal for agent_id: {agent_id}")
        else:
            logger.warning("No agent_id found in record")

        # Determine signal type
        signal_type_str = get(record, "signal_type", "").upper()
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
        initial_data = get(record, "initial_data", {})
        if isinstance(initial_data, str):
            try:
                initial_data = json.loads(initial_data)
            except json.JSONDecodeError:
                logger.error(f"Invalid JSON in initial_data: {initial_data}")
                initial_data = {}

        # Create the base request via pb2 namespace using the correct field names
        request = pb2.SignalRequest(
            signal_id=signal_id,
            agent_id=agent_id,
            signal_type=signal_type,
        )

        # Handle payload based on signal type - access via pb2 namespace
        if signal_type == pb2.SignalType.RUN:
            # Ensure the run_data has the expected structure with a "data" field
            # that the Rust engine is looking for
            run_data_wrapper = {"data": initial_data}
            request.run_data.CopyFrom(dict_to_struct(run_data_wrapper))
        elif signal_type == pb2.SignalType.SYNC:
            sync_payload = create_sync_payload(initial_data)
            request.sync.CopyFrom(sync_payload)
        elif signal_type == pb2.SignalType.FYI:
            # Wrap fyi_data in a similar structure for consistency
            fyi_data_wrapper = {"data": initial_data}
            request.fyi_data.CopyFrom(dict_to_struct(fyi_data_wrapper))

        return request
    except Exception as e:
        logger.error(f"Error creating signal request: {str(e)}")
        return None


def create_sync_payload(data: dict[str, Any]) -> Any:
    """Create a SyncPayload from the initial_data"""
    # Get sync scope
    scope_str = get(data, "scope", "ALL").upper()
    try:
        # Access via pb2 namespace
        scope = pb2.SyncScope.Value(scope_str)
    except ValueError:
        logger.error(f"Invalid scope: {scope_str}")
        # Default via pb2 namespace
        scope = pb2.SyncScope.ALL

    # Get entity UUIDs
    entity_uuids = [str(uuid_val) for uuid_val in get(data, "entity_uuids", [])]

    # Get entity types
    entity_types_str = get(data, "entity_types", [])
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
async def handle_signal_insert(payload: dict[str, Any], client: BridgeClient) -> None:
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


async def handle_agent_insert(payload: dict[str, Any], client: BridgeClient) -> None:
    """Handles a new Agent inserted in postgres"""
    # Create and send a `CreateAgentRequest`
    try:
        # Sanitize the payload before any processing
        safe_payload = sanitize_data(payload)
        logger.info(f"🔔 New agent created: {safe_payload}")

        # Extract record data from the Supabase payload using pydian get
        record = get(safe_payload, "data.record", {})

        if not record:
            logger.error("No record found in agent insert payload")
            return

        # Create the CreateAgentRequest with the agent data
        if not client.stub:
            logger.error("gRPC stub not initialized")
            return

        # Convert record to a Protobuf Struct
        agent_json_struct = dict_to_struct(record)

        # Create request
        request = pb2.CreateAgentRequest(agent_json=agent_json_struct)

        # Send request
        try:
            response = await client.stub.CreateAgent(request)
            if response and response.success:
                logger.info(f"Successfully created agent: {response.message}")
            else:
                error_msg = (
                    get(response, "message") if response else "No response received"
                )
                logger.error(f"Failed to create agent: {error_msg}")
        except Exception as e:
            logger.error(f"Error sending CreateAgentRequest: {sanitize_data(str(e))}")
    except Exception as e:
        logger.error(f"Error handling new agent: {str(e)}")


async def handle_agent_delete(payload: dict[str, Any], client: BridgeClient) -> None:
    """Handles a new Agent deleted in postgres"""
    # Create and send a `DeleteAgentRequest`
    try:
        # Sanitize the payload before any processing
        safe_payload = sanitize_data(payload)
        logger.info(f"🔔 Agent deleted: {safe_payload}")

        # Extract record data from the Supabase payload using pydian get
        record = get(safe_payload, "data.record", {})

        if not record:
            logger.error("No record found in agent delete payload")
            return

        # Get the agent ID from the record
        agent_id = get(record, "id", 0)
        if not agent_id:
            logger.error("No agent ID found in delete record")
            return

        # Create the DeleteAgentRequest
        if not client.stub:
            logger.error("gRPC stub not initialized")
            return

        # Create request
        request = pb2.DeleteAgentRequest(agent_id=agent_id)

        # Send request
        try:
            response = await client.stub.DeleteAgent(request)
            if response and response.success:
                logger.info(f"Successfully deleted agent: {response.message}")
            else:
                error_msg = (
                    get(response, "message") if response else "No response received"
                )
                logger.error(f"Failed to delete agent: {error_msg}")
        except Exception as e:
            logger.error(f"Error sending DeleteAgentRequest: {sanitize_data(str(e))}")
    except Exception as e:
        logger.error(f"Error handling agent deletion: {str(e)}")

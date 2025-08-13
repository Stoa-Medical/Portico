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

    async def plan_workflow(self, plan_request: dict[str, Any]) -> Any:
        """Send a plan workflow request to the engine"""
        try:
            if not self.stub:
                logger.error("gRPC stub not initialized")
                return None

            # Create PlanWorkflowRequest
            agent_id = plan_request.get("agent_id", 0)
            objective = plan_request.get("objective", "")
            is_ephemeral = plan_request.get("is_ephemeral", False)

            # Convert context and constraints to Protobuf Struct if they exist
            context_struct = dict_to_struct(plan_request.get("context", {}))
            constraints_struct = dict_to_struct(plan_request.get("constraints", {}))

            request = pb2.PlanWorkflowRequest(
                agent_id=agent_id,
                objective=objective,
                context=context_struct,
                constraints=constraints_struct,
                is_ephemeral=is_ephemeral,
            )

            response = await self.stub.PlanWorkflow(request)

            if response and response.success:
                logger.info(f"Workflow planned successfully: {response.message}")
                if response.workflow_uuid:
                    logger.info(f"Created workflow UUID: {response.workflow_uuid}")
            else:
                error_msg = response.message if response else "No response received"
                logger.error(f"Workflow planning failed: {error_msg}")

            return response
        except Exception as e:
            logger.error(f"Error planning workflow: {sanitize_data(str(e))}")
            return None

    async def send_signal(self, data: dict[str, Any], meta: str = "signal") -> bool:
        """Send a signal to the engine using the unified SignalRequest structure"""
        try:
            # Handle the server init case separately
            if "server-init" in data:
                response = await self.initialize_server()
                return response is not None and response.success

            # For signal data, use the new create_signal_request function
            signal_request = await create_signal_request(data)
            if signal_request:
                response = await self.process_signal(signal_request)
                if response and getattr(response, "success", False):
                    logger.info(f"Signal processed successfully: {response.message}")
                    return True
                else:
                    error_msg = getattr(response, "message", "Unknown error")
                    logger.error(f"Signal processing failed: {error_msg}")
                    return False
            else:
                logger.error("Failed to create signal request")
                return False

        except Exception as e:
            logger.error(f"Error sending signal: {sanitize_data(str(e))}")
            return False

    async def close(self) -> None:
        """Close the gRPC connection"""
        if self.channel:
            await self.channel.close()


def sanitize_data(data: Any) -> str:
    """Sanitize data for logging to prevent log injection"""
    if isinstance(data, dict):
        return json.dumps(data, indent=2, default=str)
    return str(data).replace("\n", "\\n").replace("\r", "\\r")


async def create_signal_request(data: dict[str, Any]) -> Any:
    """Create a SignalRequest from the Supabase payload"""
    try:
        # Extract record data from the Supabase payload using pydian get
        record = get(data, "record", {})

        if not record:
            logger.error("No record found in payload")
            return None

        # Extract signal ID
        signal_id = get(record, "id", 0)

        # Extract workflow_id (was agent_id)
        workflow_id = get(record, "workflow_id", 0)

        if workflow_id:
            logger.info(f"Processing signal for workflow_id: {workflow_id}")
        else:
            logger.warning("No workflow_id found in record")

        # Determine signal type
        signal_type_str = get(record, "signal_type", "").upper()
        signal_type = pb2.RUN  # Default

        if signal_type_str == "SYNC":
            signal_type = pb2.SYNC
        elif signal_type_str == "FYI":
            signal_type = pb2.FYI

        logger.info(f"Signal type: {signal_type_str}")

        # Extract payload data
        initial_data = get(record, "initial_data", {})

        # Create appropriate payload based on signal type
        payload = None
        if signal_type == pb2.RUN and initial_data:
            payload = pb2.SignalRequest(run_data=dict_to_struct(initial_data))
        elif signal_type == pb2.SYNC:
            # Create sync payload
            sync_payload = pb2.SyncPayload(
                scope=pb2.ALL,  # Default to ALL scope
                workflow_uuids=[],  # Empty for ALL scope
            )
            payload = pb2.SignalRequest(sync=sync_payload)
        elif signal_type == pb2.FYI and initial_data:
            payload = pb2.SignalRequest(fyi_data=dict_to_struct(initial_data))

        # Create the SignalRequest
        signal_request = pb2.SignalRequest(
            signal_id=signal_id,
            workflow_id=workflow_id,
            signal_type=signal_type,
        )

        # Set the appropriate payload
        if signal_type == pb2.RUN and initial_data:
            signal_request.run_data.CopyFrom(dict_to_struct(initial_data))
        elif signal_type == pb2.SYNC:
            sync_payload = pb2.SyncPayload(
                scope=pb2.ALL,
                workflow_uuids=[],
            )
            signal_request.sync.CopyFrom(sync_payload)
        elif signal_type == pb2.FYI and initial_data:
            signal_request.fyi_data.CopyFrom(dict_to_struct(initial_data))

        return signal_request

    except Exception as e:
        logger.error(f"Error creating signal request: {sanitize_data(str(e))}")
        return None


async def handle_signal_insert(payload: dict[str, Any], client: BridgeClient) -> None:
    """Handles a new Signal inserted in postgres"""
    try:
        # Sanitize the payload before any processing
        safe_payload = sanitize_data(payload)
        logger.info(f"🔔 New signal: {safe_payload}")

        # Extract record data from the Supabase payload using pydian get
        record = get(payload, "record", {})

        if not record:
            logger.error("No record found in signal payload")
            return

        # Create and send signal request
        signal_request = await create_signal_request(payload)
        if signal_request:
            response = await client.process_signal(signal_request)
            if response and response.success:
                logger.info(f"Signal processed successfully: {response.message}")
            else:
                error_msg = response.message if response else "No response received"
                logger.error(f"Signal processing failed: {error_msg}")
        else:
            logger.error("Failed to create signal request")

    except Exception as e:
        logger.error(f"Error handling signal: {str(e)}")


async def handle_workflow_insert(payload: dict[str, Any], client: BridgeClient) -> None:
    """Handles a new Workflow inserted in postgres"""
    # Create and send a `CreateWorkflowRequest`
    try:
        # Sanitize the payload before any processing
        safe_payload = sanitize_data(payload)
        logger.info(f"🔔 New workflow created: {safe_payload}")

        # Extract record data from the Supabase payload using pydian get
        record = get(safe_payload, "data.record", {})

        if not record:
            logger.error("No record found in workflow insert payload")
            return

        # Create the CreateWorkflowRequest with the workflow data
        if not client.stub:
            logger.error("gRPC stub not initialized")
            return

        # Convert record to a Protobuf Struct
        workflow_json_struct = dict_to_struct(record)

        # Create request
        request = pb2.CreateWorkflowRequest(workflow_json=workflow_json_struct)

        # Send request
        try:
            response = await client.stub.CreateWorkflow(request)
            if response and response.success:
                logger.info(f"Successfully created workflow: {response.message}")
            else:
                error_msg = (
                    get(response, "message") if response else "No response received"
                )
                logger.error(f"Failed to create workflow: {error_msg}")
        except Exception as e:
            logger.error(
                f"Error sending CreateWorkflowRequest: {sanitize_data(str(e))}"
            )
    except Exception as e:
        logger.error(f"Error handling new workflow: {str(e)}")


async def handle_workflow_delete(payload: dict[str, Any], client: BridgeClient) -> None:
    """Handles a new Workflow deleted in postgres"""
    # Create and send a `DeleteWorkflowRequest`
    try:
        # Sanitize the payload before any processing
        safe_payload = sanitize_data(payload)
        logger.info(f"🔔 Workflow deleted: {safe_payload}")

        # Extract record data from the Supabase payload using pydian get
        record = get(safe_payload, "data.record", {})

        if not record:
            logger.error("No record found in workflow delete payload")
            return

        # Get the workflow ID from the record
        workflow_id = get(record, "id", 0)
        if not workflow_id:
            logger.error("No workflow ID found in delete record")
            return

        # Create the DeleteWorkflowRequest
        if not client.stub:
            logger.error("gRPC stub not initialized")
            return

        # Create request
        request = pb2.DeleteWorkflowRequest(workflow_id=workflow_id)

        # Send request
        try:
            response = await client.stub.DeleteWorkflow(request)
            if response and response.success:
                logger.info(f"Successfully deleted workflow: {response.message}")
            else:
                error_msg = (
                    get(response, "message") if response else "No response received"
                )
                logger.error(f"Failed to delete workflow: {error_msg}")
        except Exception as e:
            logger.error(
                f"Error sending DeleteWorkflowRequest: {sanitize_data(str(e))}"
            )
    except Exception as e:
        logger.error(f"Error handling workflow deletion: {str(e)}")


async def handle_agent_composition_request(
    payload: dict[str, Any], client: BridgeClient
) -> None:
    """Handles an agent composition request to plan and create workflows"""
    try:
        # Sanitize the payload before any processing
        safe_payload = sanitize_data(payload)
        logger.info(f"🧠 Agent composition request: {safe_payload}")

        # Extract request data from the payload
        record = get(payload, "record", {})

        if not record:
            logger.error("No record found in agent composition payload")
            return

        # Extract required fields for planning
        agent_id = get(record, "agent_id")
        objective = get(record, "objective", "")
        context = get(record, "context", {})
        constraints = get(record, "constraints", {})
        is_ephemeral = get(record, "is_ephemeral", False)

        if not agent_id or not objective:
            logger.error("Missing required fields: agent_id or objective")
            return

        # Create plan request
        plan_request = {
            "agent_id": agent_id,
            "objective": objective,
            "context": context,
            "constraints": constraints,
            "is_ephemeral": is_ephemeral,
        }

        # Send plan request to engine
        response = await client.plan_workflow(plan_request)

        if response and response.success:
            logger.info(
                f"Agent {agent_id} successfully planned workflow: {response.message}"
            )

            # If a workflow was created, log additional details
            if hasattr(response, "workflow_uuid") and response.workflow_uuid:
                logger.info(f"New workflow created with UUID: {response.workflow_uuid}")

            if hasattr(response, "estimated_steps"):
                logger.info(f"Estimated steps: {response.estimated_steps}")

            if hasattr(response, "requires_approval") and response.requires_approval:
                logger.info("Workflow requires approval before execution")

        else:
            error_msg = response.message if response else "No response received"
            logger.error(f"Agent composition failed: {error_msg}")

            # Log validation errors if present
            if hasattr(response, "validation_errors") and response.validation_errors:
                for error in response.validation_errors:
                    logger.error(f"Validation error: {error}")

    except Exception as e:
        logger.error(f"Error handling agent composition request: {str(e)}")


async def handle_agent_insert(payload: dict[str, Any], client: BridgeClient) -> None:
    """Handles a new Agent inserted in postgres"""
    try:
        # Sanitize the payload before any processing
        safe_payload = sanitize_data(payload)
        logger.info(f"🤖 New agent created: {safe_payload}")

        # Extract record data from the Supabase payload
        record = get(payload, "record", {})

        if not record:
            logger.error("No record found in agent insert payload")
            return

        agent_id = get(record, "id")
        agent_name = get(record, "name", "Unknown")
        capabilities = get(record, "capabilities_json", {})
        policy = get(record, "policy_json", {})

        logger.info(
            f"Agent {agent_id} ({agent_name}) registered with capabilities: {capabilities}"
        )
        logger.info(f"Agent {agent_id} policy: {policy}")

        # In a real implementation, you might want to:
        # 1. Cache agent capabilities for faster lookup
        # 2. Validate agent configuration
        # 3. Initialize agent-specific resources

    except Exception as e:
        logger.error(f"Error handling agent insert: {str(e)}")


async def handle_agent_update(payload: dict[str, Any], client: BridgeClient) -> None:
    """Handles an Agent update in postgres"""
    try:
        # Sanitize the payload before any processing
        safe_payload = sanitize_data(payload)
        logger.info(f"🔄 Agent updated: {safe_payload}")

        # Extract record data from the Supabase payload
        record = get(payload, "record", {})

        if not record:
            logger.error("No record found in agent update payload")
            return

        agent_id = get(record, "id")
        agent_name = get(record, "name", "Unknown")

        logger.info(f"Agent {agent_id} ({agent_name}) configuration updated")

        # In a real implementation, you might want to:
        # 1. Update cached agent capabilities
        # 2. Revalidate existing workflows for this agent
        # 3. Notify running sessions of capability changes

    except Exception as e:
        logger.error(f"Error handling agent update: {str(e)}")

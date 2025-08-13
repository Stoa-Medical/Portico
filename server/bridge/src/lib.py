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


# Helper functions for enhanced workflow handling
def get_current_timestamp() -> str:
    """Get current timestamp in ISO format"""
    from datetime import datetime

    return datetime.utcnow().isoformat()


async def store_workflow_metadata(metadata: dict[str, Any]) -> None:
    """Store workflow metadata for tracking (placeholder implementation)"""
    try:
        # In production, this would store to database
        logger.info(f"Storing workflow metadata: {sanitize_data(metadata)}")

        # TODO: Implement actual database storage
        # Example: INSERT INTO workflow_metadata (agent_id, workflow_uuid, ...) VALUES (...)

    except Exception as e:
        logger.error(f"Failed to store workflow metadata: {e}")


async def store_failure_metadata(metadata: dict[str, Any]) -> None:
    """Store failure metadata for analysis (placeholder implementation)"""
    try:
        # In production, this would store to database
        logger.error(f"Storing failure metadata: {sanitize_data(metadata)}")

        # TODO: Implement actual database storage for failure tracking
        # This helps with debugging and improving agent planning

    except Exception as e:
        logger.error(f"Failed to store failure metadata: {e}")


async def handle_approval_required(workflow_metadata: dict[str, Any]) -> None:
    """Handle workflows that require approval (placeholder implementation)"""
    try:
        logger.info(
            f"Workflow {workflow_metadata.get('workflow_uuid', 'unknown')} requires approval"
        )

        # TODO: Implement approval system integration
        # Examples:
        # - Send notification to administrators
        # - Store in approval queue
        # - Integrate with external approval systems
        # - Send webhook to frontend for user approval

        # For now, just log the requirement
        agent_id = workflow_metadata.get("agent_id")
        objective = workflow_metadata.get("objective")
        logger.info(f"Agent {agent_id} workflow '{objective}' queued for approval")

    except Exception as e:
        logger.error(f"Failed to handle approval requirement: {e}")


async def trigger_workflow_execution(workflow_uuid: str, agent_id: int) -> None:
    """Trigger automatic execution of approved workflows (placeholder implementation)"""
    try:
        logger.info(
            f"Triggering execution of workflow {workflow_uuid} for agent {agent_id}"
        )

        # TODO: Implement actual workflow execution triggering
        # This would typically:
        # 1. Create a RUN signal for the workflow
        # 2. Send it to the engine for execution
        # 3. Monitor execution status

        # For now, just log the trigger
        logger.info(f"Auto-execution triggered for workflow {workflow_uuid}")

    except Exception as e:
        logger.error(f"Failed to trigger workflow execution: {e}")


async def process_agent_registration(
    agent_id: int,
    agent_name: str,
    capabilities: dict,
    policy: dict,
    client: BridgeClient,
) -> None:
    """Process new agent registration with enhanced functionality"""
    try:
        logger.info(f"Processing registration for agent {agent_id} ({agent_name})")

        # Analyze agent capabilities for optimization
        capability_analysis = analyze_agent_capabilities(capabilities)
        logger.info(f"Agent {agent_id} capability analysis: {capability_analysis}")

        # Check for security policy compliance
        security_check = validate_security_policy(policy)
        if not security_check["compliant"]:
            logger.warning(
                f"Agent {agent_id} has security policy concerns: {security_check['issues']}"
            )

        # Initialize agent-specific resources if needed
        await initialize_agent_resources(agent_id, capabilities)

        # Notify relevant systems about new agent
        await notify_agent_registered(agent_id, agent_name, capabilities)

    except Exception as e:
        logger.error(f"Failed to process agent registration: {e}")


def validate_agent_configuration(capabilities: dict, policy: dict) -> dict:
    """Validate agent configuration and return validation results"""
    warnings = []
    is_valid = True

    # Check capabilities structure
    tools = capabilities.get("tools", [])
    models = capabilities.get("models", [])

    if not tools and not models:
        warnings.append("Agent has no tools or models configured")

    # Check for dangerous tool combinations
    if "python" in tools and "webscrape" in tools:
        warnings.append(
            "Agent has both Python and webscraping capabilities - ensure security policies are configured"
        )

    # Validate policy constraints
    max_steps = capabilities.get("max_steps")
    if max_steps and max_steps > 50:
        warnings.append(
            f"Agent allows up to {max_steps} steps which may impact performance"
        )

    max_workflows = policy.get("max_workflows_per_hour")
    if max_workflows and max_workflows > 1000:
        warnings.append(
            f"Agent allows {max_workflows} workflows/hour which may impact system performance"
        )

    return {"is_valid": is_valid, "warnings": warnings}


def analyze_agent_capabilities(capabilities: dict) -> dict:
    """Analyze agent capabilities and provide insights"""
    tools = capabilities.get("tools", [])
    models = capabilities.get("models", [])
    max_steps = capabilities.get("max_steps", 0)
    can_create_ephemeral = capabilities.get("can_create_ephemeral", False)

    analysis = {
        "tool_count": len(tools),
        "model_count": len(models),
        "max_steps": max_steps,
        "can_create_ephemeral": can_create_ephemeral,
        "complexity_level": "basic",
    }

    # Determine complexity level
    if len(tools) > 3 or len(models) > 2:
        analysis["complexity_level"] = "advanced"
    elif len(tools) > 1 or len(models) > 1:
        analysis["complexity_level"] = "intermediate"

    # Check for specialized capabilities
    specialized_tools = ["python", "webscrape", "database"]
    has_specialized = any(tool in tools for tool in specialized_tools)
    analysis["has_specialized_tools"] = has_specialized

    return analysis


def validate_security_policy(policy: dict) -> dict:
    """Validate security policy configuration"""
    issues = []

    security_constraints = policy.get("security_constraints", {})
    if not security_constraints:
        issues.append("No security constraints configured")

    allowed_patterns = policy.get("allowed_patterns", [])
    if not allowed_patterns:
        issues.append(
            "No allowed patterns specified - agent can create any workflow type"
        )

    return {"compliant": len(issues) == 0, "issues": issues}


async def cache_agent_info(agent_id: int, info: dict) -> None:
    """Cache agent information for performance (placeholder implementation)"""
    try:
        # In production, this would use Redis or similar
        logger.info(f"Caching info for agent {agent_id}")
        # TODO: Implement actual caching
    except Exception as e:
        logger.error(f"Failed to cache agent info: {e}")


async def initialize_agent_resources(agent_id: int, capabilities: dict) -> None:
    """Initialize any resources needed for the agent (placeholder implementation)"""
    try:
        logger.info(f"Initializing resources for agent {agent_id}")
        # TODO: Initialize agent-specific resources like queues, metrics, etc.
    except Exception as e:
        logger.error(f"Failed to initialize agent resources: {e}")


async def notify_agent_registered(
    agent_id: int, agent_name: str, capabilities: dict
) -> None:
    """Notify other systems about agent registration (placeholder implementation)"""
    try:
        logger.info(f"Notifying systems about agent {agent_id} registration")
        # TODO: Send notifications to monitoring, UI, etc.
    except Exception as e:
        logger.error(f"Failed to notify about agent registration: {e}")


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

            # Store comprehensive workflow metadata
            workflow_metadata = {
                "agent_id": agent_id,
                "objective": objective,
                "context": context,
                "constraints": constraints,
                "is_ephemeral": is_ephemeral,
                "planning_timestamp": get_current_timestamp(),
            }

            # If a workflow was created, log additional details and store metadata
            if hasattr(response, "workflow_uuid") and response.workflow_uuid:
                workflow_metadata["workflow_uuid"] = response.workflow_uuid
                logger.info(f"New workflow created with UUID: {response.workflow_uuid}")

            if hasattr(response, "estimated_steps"):
                workflow_metadata["estimated_steps"] = response.estimated_steps
                logger.info(f"Estimated steps: {response.estimated_steps}")

            if hasattr(response, "requires_approval") and response.requires_approval:
                workflow_metadata["requires_approval"] = True
                logger.info("Workflow requires approval before execution")

                # TODO: Implement approval workflow system
                # This would integrate with a notification system or approval queue
                await handle_approval_required(workflow_metadata)
            else:
                workflow_metadata["requires_approval"] = False

            # Store workflow spec if available for future reference
            if hasattr(response, "workflow_spec") and response.workflow_spec:
                workflow_metadata["workflow_spec"] = response.workflow_spec
                logger.info("Workflow specification stored for future execution")

            # Store workflow metadata in database or cache for tracking
            await store_workflow_metadata(workflow_metadata)

            # If auto-execute is enabled and no approval required, trigger execution
            auto_execute = get(record, "auto_execute", False)
            if auto_execute and not workflow_metadata.get("requires_approval", False):
                await trigger_workflow_execution(response.workflow_uuid, agent_id)

        else:
            error_msg = response.message if response else "No response received"
            logger.error(f"Agent composition failed: {error_msg}")

            # Store failure metadata for analysis
            failure_metadata = {
                "agent_id": agent_id,
                "objective": objective,
                "error_message": error_msg,
                "timestamp": get_current_timestamp(),
            }

            # Log validation errors if present
            if hasattr(response, "validation_errors") and response.validation_errors:
                failure_metadata["validation_errors"] = response.validation_errors
                for error in response.validation_errors:
                    logger.error(f"Validation error: {error}")

            await store_failure_metadata(failure_metadata)

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

        # Enhanced agent registration processing
        await process_agent_registration(
            agent_id, agent_name, capabilities, policy, client
        )

        # Validate agent configuration
        validation_result = validate_agent_configuration(capabilities, policy)
        if not validation_result["is_valid"]:
            logger.warning(
                f"Agent {agent_id} configuration issues: {validation_result['warnings']}"
            )

        # Cache agent information for performance
        await cache_agent_info(
            agent_id,
            {
                "name": agent_name,
                "capabilities": capabilities,
                "policy": policy,
                "registration_timestamp": get_current_timestamp(),
            },
        )

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

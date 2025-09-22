"""
gRPC client for communicating with the Portico engine service.
"""

from __future__ import annotations
import grpc
from typing import Any
from .utils import logger, sanitize_data
from .converters import dict_to_struct

# Import the generated gRPC code
from src.proto import bridge_message_pb2 as pb2
from src.proto import bridge_message_pb2_grpc as pb2_grpc


class BridgeClient:
    """gRPC client for communicating with the engine service"""

    def __init__(self, host: str, port: int) -> None:
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

    async def initialize_server(self) -> pb2.GeneralResponse | None:  # type: ignore[name-defined]
        """Initialize the server connection"""
        try:
            if not self.stub:
                logger.error("gRPC stub not initialized")
                return None

            request = pb2.ServerInitRequest(server_init=True)  # type: ignore[attr-defined]
            response = await self.stub.InitServer(request)
            return response
        except Exception as e:
            logger.error(f"Error initializing server: {e}")
            return None

    async def process_signal(
        self, signal_request: pb2.SignalRequest  # type: ignore[name-defined]
    ) -> pb2.SignalResponse | None:  # type: ignore[name-defined]
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

            return response
        except Exception as e:
            logger.error(f"Error processing signal: {sanitize_data(str(e))}")
            return None

    async def plan_workflow(
        self, plan_request: dict[str, Any]
    ) -> pb2.PlanWorkflowResponse | None:  # type: ignore[name-defined]
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

            request = pb2.PlanWorkflowRequest(  # type: ignore[attr-defined]
                agent_id=agent_id,
                objective=objective,
                context=context_struct,
                constraints=constraints_struct,
                is_ephemeral=is_ephemeral,
            )

            response = await self.stub.PlanWorkflow(request)  # type: ignore[attr-defined]

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

    async def create_workflow(
        self, workflow_json: dict[str, Any]
    ) -> pb2.GeneralResponse | None:  # type: ignore[name-defined]
        """Send a create workflow request to the engine"""
        try:
            if not self.stub:
                logger.error("gRPC stub not initialized")
                return None

            # Convert record to a Protobuf Struct
            workflow_json_struct = dict_to_struct(workflow_json)

            # Create request
            request = pb2.CreateWorkflowRequest(workflow_json=workflow_json_struct)  # type: ignore[attr-defined]

            # Send request
            response = await self.stub.CreateWorkflow(request)
            return response
        except Exception as e:
            logger.error(
                f"Error sending CreateWorkflowRequest: {sanitize_data(str(e))}"
            )
            return None

    async def delete_workflow(self, workflow_id: int) -> pb2.GeneralResponse | None:  # type: ignore[name-defined]
        """Send a delete workflow request to the engine"""
        try:
            if not self.stub:
                logger.error("gRPC stub not initialized")
                return None

            # Create request
            request = pb2.DeleteWorkflowRequest(workflow_id=workflow_id)  # type: ignore[attr-defined]

            # Send request
            response = await self.stub.DeleteWorkflow(request)
            return response
        except Exception as e:
            logger.error(
                f"Error sending DeleteWorkflowRequest: {sanitize_data(str(e))}"
            )
            return None

    async def close(self) -> None:
        """Close the gRPC connection"""
        if self.channel:
            await self.channel.close()

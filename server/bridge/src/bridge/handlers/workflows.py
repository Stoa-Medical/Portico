"""
Workflow event handlers for the bridge service.
"""

from __future__ import annotations
from typing import Any
from pydian import get

from ..grpc_client import BridgeClient
from ..utils import logger, sanitize_data


async def handle_workflow_insert(payload: dict[str, Any], client: BridgeClient) -> None:
    """Handles a new Workflow inserted in postgres"""
    try:
        # Sanitize the payload before any processing
        safe_payload = sanitize_data(payload)
        logger.info(f"🔔 New workflow created: {safe_payload}")

        # Extract record data from the Supabase payload using pydian get
        record = get(payload, "record", {})

        if not record:
            logger.error("No record found in workflow insert payload")
            return

        # Create and send the CreateWorkflowRequest
        response = await client.create_workflow(record)
        if response and response.success:
            logger.info(f"Successfully created workflow: {response.message}")
        else:
            error_msg = response.message if response else "No response received"
            logger.error(f"Failed to create workflow: {error_msg}")

    except Exception as e:
        logger.error(f"Error handling new workflow: {str(e)}")


async def handle_workflow_delete(payload: dict[str, Any], client: BridgeClient) -> None:
    """Handles a new Workflow deleted in postgres"""
    try:
        # Sanitize the payload before any processing
        safe_payload = sanitize_data(payload)
        logger.info(f"🔔 Workflow deleted: {safe_payload}")

        # Extract record data from the Supabase payload using pydian get
        record = get(payload, "record", {})

        if not record:
            logger.error("No record found in workflow delete payload")
            return

        # Get the workflow ID from the record
        workflow_id = get(record, "id", 0)
        if not workflow_id:
            logger.error("No workflow ID found in delete record")
            return

        # Create and send the DeleteWorkflowRequest
        response = await client.delete_workflow(workflow_id)
        if response and response.success:
            logger.info(f"Successfully deleted workflow: {response.message}")
        else:
            error_msg = response.message if response else "No response received"
            logger.error(f"Failed to delete workflow: {error_msg}")

    except Exception as e:
        logger.error(f"Error handling workflow deletion: {str(e)}")

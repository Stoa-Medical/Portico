"""
Agent event handlers for the bridge service.
"""

from __future__ import annotations
from typing import Any
from pydian import get

from ..grpc_client import BridgeClient
from ..utils import logger, sanitize_data, get_current_timestamp
from ..metadata import (
    store_workflow_metadata,
    store_failure_metadata,
    cache_agent_info,
    initialize_agent_resources,
    notify_agent_registered,
)


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


async def trigger_workflow_execution(
    workflow_uuid: str, agent_id: int, client: BridgeClient
) -> None:
    """Trigger automatic execution of approved workflows"""
    try:
        logger.info(
            f"Triggering execution of workflow {workflow_uuid} for agent {agent_id}"
        )

        # Create a RUN signal for the workflow
        run_signal_data = {
            "record": {
                "id": f"auto_run_{workflow_uuid}",
                "workflow_id": agent_id,
                "signal_type": "RUN",
                "initial_data": {
                    "workflow_uuid": workflow_uuid,
                    "auto_triggered": True,
                    "trigger_reason": "auto_execution_after_planning",
                },
            }
        }

        # Import here to avoid circular imports
        from ..converters import create_signal_request

        # Create and send the RUN signal
        signal_request = create_signal_request(run_signal_data)
        if signal_request:
            response = await client.process_signal(signal_request)
            if response and response.success:
                logger.info(
                    f"Successfully triggered execution for workflow {workflow_uuid}: {response.message}"
                )

                # Store execution trigger metadata
                trigger_metadata = {
                    "workflow_uuid": workflow_uuid,
                    "agent_id": agent_id,
                    "trigger_timestamp": get_current_timestamp(),
                    "trigger_type": "auto_execution",
                    "success": True,
                }
                await store_workflow_metadata(trigger_metadata)

            else:
                error_msg = response.message if response else "No response received"
                logger.error(f"Failed to trigger workflow execution: {error_msg}")

                # Store failure metadata
                failure_metadata = {
                    "workflow_uuid": workflow_uuid,
                    "agent_id": agent_id,
                    "error_message": error_msg,
                    "trigger_type": "auto_execution_failed",
                    "timestamp": get_current_timestamp(),
                }
                await store_failure_metadata(failure_metadata)
        else:
            logger.error(f"Failed to create RUN signal for workflow {workflow_uuid}")

    except Exception as e:
        logger.error(f"Failed to trigger workflow execution: {e}")


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
                await trigger_workflow_execution(
                    response.workflow_uuid, agent_id, client
                )

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
            if response and hasattr(response, "validation_errors") and response.validation_errors:  # type: ignore[union-attr]
                failure_metadata["validation_errors"] = response.validation_errors  # type: ignore[union-attr]
                for error in response.validation_errors:  # type: ignore[union-attr]
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

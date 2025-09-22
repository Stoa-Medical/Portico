"""
Event handlers for the bridge service.

This package contains handlers organized by domain:
- signals: Signal event handling
- workflows: Workflow lifecycle events
- agents: Agent management events
"""

from .signals import handle_signal_insert
from .workflows import handle_workflow_insert, handle_workflow_delete
from .agents import (
    handle_agent_insert,
    handle_agent_update,
    handle_agent_composition_request,
)

__all__ = [
    "handle_signal_insert",
    "handle_workflow_insert",
    "handle_workflow_delete",
    "handle_agent_insert",
    "handle_agent_update",
    "handle_agent_composition_request",
]

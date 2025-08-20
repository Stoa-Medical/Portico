"""
Data conversion utilities for transforming between Python dicts and Protocol Buffer messages.
"""

from __future__ import annotations
from typing import Any
from google.protobuf.struct_pb2 import Struct
from google.protobuf.json_format import ParseDict
from pydian import get
from .utils import logger

# Import the generated gRPC code
from src.proto import bridge_message_pb2 as pb2


def dict_to_struct(data: dict[str, Any]) -> Struct:
    """Convert a Python dictionary to a Protobuf Struct"""
    struct = Struct()
    ParseDict(data, struct)
    return struct


def create_signal_request(data: dict[str, Any]) -> pb2.SignalRequest | None:  # type: ignore[name-defined]
    """Create a SignalRequest from the Supabase payload"""
    try:
        # Extract record data from the Supabase payload using pydian get
        record = get(data, "record", {})

        if not record:
            logger.error("No record found in payload")
            return None

        # Extract signal ID
        signal_id = get(record, "id", 0)

        # Extract workflow_id
        workflow_id = get(record, "workflow_id", 0)

        if workflow_id:
            logger.info(f"Processing signal for workflow_id: {workflow_id}")
        else:
            logger.warning("No workflow_id found in record")

        # Determine signal type
        signal_type_str = get(record, "signal_type", "").upper()
        signal_type = pb2.RUN  # type: ignore[attr-defined]  # Default

        if signal_type_str == "SYNC":
            signal_type = pb2.SYNC  # type: ignore[attr-defined]
        elif signal_type_str == "FYI":
            signal_type = pb2.FYI  # type: ignore[attr-defined]

        logger.info(f"Signal type: {signal_type_str}")

        # Extract payload data
        initial_data = get(record, "initial_data", {})

        # Create the SignalRequest
        signal_request = pb2.SignalRequest(  # type: ignore[attr-defined]
            signal_id=signal_id,
            workflow_id=workflow_id,
            signal_type=signal_type,
        )

        # Set the appropriate payload based on signal type
        if signal_type == pb2.RUN and initial_data:  # type: ignore[attr-defined]
            signal_request.run_data.CopyFrom(dict_to_struct(initial_data))
        elif signal_type == pb2.SYNC:  # type: ignore[attr-defined]
            sync_payload = pb2.SyncPayload(  # type: ignore[attr-defined]
                scope=pb2.ALL,  # type: ignore[attr-defined]
                workflow_uuids=[],
            )
            signal_request.sync.CopyFrom(sync_payload)
        elif signal_type == pb2.FYI and initial_data:  # type: ignore[attr-defined]
            signal_request.fyi_data.CopyFrom(dict_to_struct(initial_data))

        return signal_request

    except Exception as e:
        logger.error(f"Error creating signal request: {str(e)}")
        return None

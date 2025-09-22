"""
Signal event handlers for the bridge service.
"""

from __future__ import annotations
from typing import Any
from pydian import get

from ..grpc_client import BridgeClient
from ..converters import create_signal_request
from ..utils import logger, sanitize_data


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
        signal_request = create_signal_request(payload)
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

"""
Shared utilities for the bridge service.
"""

from __future__ import annotations
import json
import logging
from datetime import datetime
from typing import Any

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("portico-bridge")


def get_current_timestamp() -> str:
    """Get current timestamp in ISO format"""
    return datetime.utcnow().isoformat()


def sanitize_data(data: Any) -> str:
    """Sanitize data for logging to prevent log injection"""
    if isinstance(data, dict):
        return json.dumps(data, indent=2, default=str)
    return str(data).replace("\n", "\\n").replace("\r", "\\r")

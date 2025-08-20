"""
Bridge service modules for Portico.

This package contains the refactored bridge service components:
- grpc_client: gRPC communication interface
- converters: Data transformation utilities
- handlers: Event handlers organized by domain
- utils: Shared utilities and logging
- metadata: Metadata storage functionality
"""

from .grpc_client import BridgeClient
from .converters import create_signal_request, dict_to_struct
from .utils import logger, sanitize_data, get_current_timestamp

__all__ = [
    "BridgeClient",
    "create_signal_request",
    "dict_to_struct",
    "logger",
    "sanitize_data",
    "get_current_timestamp",
]

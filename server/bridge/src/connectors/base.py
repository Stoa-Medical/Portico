from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any, AsyncIterator
import hashlib
import json


@dataclass
class ConnectorConfig:
    """Configuration for a connector instance."""

    base_url: str | None = None
    auth: dict[str, Any] = field(default_factory=dict)
    polling_interval_secs: int = 300  # 5 minutes default
    resources: list[str] = field(default_factory=list)
    extra: dict[str, Any] = field(default_factory=dict)


@dataclass
class ConnectorEvent:
    """Event emitted when connector detects data changes."""

    connector_id: int
    connector_name: str
    resource_type: str
    action: str  # "created" | "updated" | "deleted"
    resource_id: str
    data: dict[str, Any]
    timestamp: datetime
    raw: dict[str, Any] | None = None

    @property
    def data_hash(self) -> str:
        """SHA256 hash for deduplication."""
        content = json.dumps(self.data, sort_keys=True)
        return hashlib.sha256(content.encode()).hexdigest()


class Connector(ABC):
    """Base class for all data source connectors."""

    def __init__(
        self,
        connector_id: int,
        name: str,
        config: ConnectorConfig,
        schema_mapping: dict[str, Any] | None = None,
    ):
        self.connector_id = connector_id
        self.name = name
        self.config = config
        self.schema_mapping = schema_mapping or {}
        self._running = False
        self._last_poll: datetime | None = None

    @property
    @abstractmethod
    def connector_type(self) -> str:
        """Return connector type identifier (e.g., 'fhir', 'http')."""
        pass

    @abstractmethod
    async def connect(self) -> None:
        """Establish connection to the data source."""
        pass

    @abstractmethod
    async def disconnect(self) -> None:
        """Clean up and close connection."""
        pass

    @abstractmethod
    async def poll(self) -> list[ConnectorEvent]:
        """
        Poll for new/changed data since last poll.
        Called on polling_interval_secs schedule.
        """
        pass

    @abstractmethod
    async def fetch_resource(
        self,
        resource_type: str,
        resource_id: str,
    ) -> dict[str, Any]:
        """Fetch a specific resource by type and ID."""
        pass

    async def watch(self) -> AsyncIterator[ConnectorEvent]:
        """
        Stream events in real-time (for connectors with webhook support).
        Default implementation raises NotImplementedError.
        """
        raise NotImplementedError(
            f"{self.__class__.__name__} does not support real-time streaming"
        )
        # This yield is unreachable but needed for type checking
        yield  # type: ignore

    @property
    @abstractmethod
    def supported_resources(self) -> list[str]:
        """List of resource types this connector can handle."""
        pass

    def apply_mapping(self, raw_data: dict[str, Any]) -> dict[str, Any]:
        """Apply schema mapping to transform raw data to common schema."""
        if not self.schema_mapping:
            return raw_data

        from jsonpath_ng import parse

        result = {}
        for mapping in self.schema_mapping.get("mappings", []):
            source_path = mapping["source"]
            target_field = mapping["target"]

            try:
                jsonpath_expr = parse(source_path)
                matches = jsonpath_expr.find(raw_data)
                if matches:
                    result[target_field] = matches[0].value
            except Exception:
                # Skip fields that don't match
                pass

        return result

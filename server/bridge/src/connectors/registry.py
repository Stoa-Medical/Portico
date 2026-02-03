from typing import Type

from .base import Connector, ConnectorConfig
from .fhir import FHIRConnector


class ConnectorRegistry:
    """Registry of available connector types."""

    _connectors: dict[str, Type[Connector]] = {
        "fhir": FHIRConnector,
    }

    @classmethod
    def register(cls, connector_type: str, connector_class: Type[Connector]) -> None:
        """Register a new connector type."""
        cls._connectors[connector_type] = connector_class

    @classmethod
    def get(cls, connector_type: str) -> Type[Connector] | None:
        """Get a connector class by type."""
        return cls._connectors.get(connector_type)

    @classmethod
    def list_types(cls) -> list[str]:
        """List all registered connector types."""
        return list(cls._connectors.keys())

    @classmethod
    def create(
        cls,
        connector_type: str,
        connector_id: int,
        name: str,
        config: dict,
        schema_mapping: dict | None = None,
    ) -> Connector:
        """Create a connector instance from configuration."""
        connector_class = cls.get(connector_type)
        if not connector_class:
            raise ValueError(f"Unknown connector type: {connector_type}")

        cfg = ConnectorConfig(
            base_url=config.get("base_url"),
            auth=config.get("auth", {}),
            polling_interval_secs=config.get("polling_interval_secs", 300),
            resources=config.get("resources", []),
            extra=config.get("extra", {}),
        )

        return connector_class(
            connector_id=connector_id,
            name=name,
            config=cfg,
            schema_mapping=schema_mapping,
        )

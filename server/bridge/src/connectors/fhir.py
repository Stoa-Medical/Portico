from typing import Any

from .base import Connector, ConnectorConfig, ConnectorEvent


class FHIRConnector(Connector):
    """FHIR R4 connector for EHR integration."""

    FHIR_RESOURCES = [
        "Patient",
        "Encounter",
        "ServiceRequest",
        "DocumentReference",
        "Observation",
        "Condition",
        "MedicationRequest",
        "DiagnosticReport",
    ]

    def __init__(
        self,
        connector_id: int,
        name: str,
        config: ConnectorConfig,
        schema_mapping: dict[str, Any] | None = None,
    ):
        super().__init__(connector_id, name, config, schema_mapping)

    @property
    def connector_type(self) -> str:
        return "fhir"

    @property
    def supported_resources(self) -> list[str]:
        return [r for r in self.FHIR_RESOURCES if r in self.config.resources]

    async def connect(self) -> None:
        """Establish connection to the FHIR server."""
        raise NotImplementedError("FHIRConnector.connect not yet implemented")

    async def disconnect(self) -> None:
        """Close connection to the FHIR server."""
        raise NotImplementedError("FHIRConnector.disconnect not yet implemented")

    async def poll(self) -> list[ConnectorEvent]:
        """Poll for resources updated since last poll."""
        raise NotImplementedError("FHIRConnector.poll not yet implemented")

    async def fetch_resource(
        self,
        resource_type: str,
        resource_id: str,
    ) -> dict[str, Any]:
        """Fetch a specific FHIR resource."""
        raise NotImplementedError("FHIRConnector.fetch_resource not yet implemented")

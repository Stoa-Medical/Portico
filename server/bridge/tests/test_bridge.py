import uuid
import pytest
from unittest.mock import AsyncMock, MagicMock, patch

from src.bridge import BridgeClient
from src.bridge.converters import create_signal_request
from src.proto import bridge_message_pb2 as pb2


@pytest.fixture
def signal_data():
    """Sample signal data for testing"""
    return {
        "record": {
            "id": 1,
            "workflow_id": 123,
            "signal_type": "RUN",
            "initial_data": {
                "operation": "CREATE",
                "entity_type": "AGENT",
                "entity_uuid": str(uuid.uuid4()),
                "data": {"name": "Test Agent", "description": "Test description"},
            },
        }
    }


@pytest.fixture
def sync_data():
    """Sample sync data for testing"""
    return {
        "record": {
            "id": 2,
            "workflow_id": 124,
            "signal_type": "SYNC",
            "initial_data": {"scope": "ALL", "entity_types": ["AGENT", "STEP"]},
        }
    }


@pytest.fixture
def fyi_data():
    """Sample FYI data for testing"""
    return {
        "record": {
            "id": 3,
            "workflow_id": 125,
            "signal_type": "FYI",
            "initial_data": {"message": "Test FYI", "data": {"some": "value"}},
        }
    }


# Removed create_sync_payload test as it's now handled internally


@pytest.mark.asyncio
async def test_create_signal_request_run(signal_data):
    """Test creating a run signal request"""
    request = create_signal_request(signal_data)

    assert request is not None
    assert request.signal_type == pb2.RUN
    assert request.signal_id == signal_data["record"]["id"]
    assert request.workflow_id == signal_data["record"]["workflow_id"]
    assert request.run_data is not None
    # Verify the data was serialized correctly
    initial_data = signal_data["record"]["initial_data"]
    assert "operation" in initial_data


@pytest.mark.asyncio
async def test_create_signal_request_sync(sync_data):
    """Test creating a sync signal request"""
    request = create_signal_request(sync_data)

    assert request is not None
    assert request.signal_type == pb2.SYNC
    assert request.signal_id == sync_data["record"]["id"]
    assert request.workflow_id == sync_data["record"]["workflow_id"]
    assert request.sync.scope == pb2.ALL
    assert len(request.sync.workflow_uuids) == 0


@pytest.mark.asyncio
async def test_create_signal_request_fyi(fyi_data):
    """Test creating an FYI signal request"""
    request = create_signal_request(fyi_data)

    assert request is not None
    assert request.signal_type == pb2.FYI
    assert request.signal_id == fyi_data["record"]["id"]
    assert request.workflow_id == fyi_data["record"]["workflow_id"]
    # FYI data is stored in a Struct
    assert request.fyi_data is not None


@pytest.mark.asyncio
async def test_bridge_client():
    """Test BridgeClient"""
    # Create a mock gRPC stub
    mock_stub = AsyncMock()
    mock_init_response = MagicMock()
    mock_init_response.success = True
    mock_init_response.message = "Initialized"

    mock_signal_response = MagicMock()
    mock_signal_response.success = True
    mock_signal_response.message = "Processed"
    mock_signal_response.runtime_session_uuid = str(uuid.uuid4())

    # Set up the mock stub methods
    mock_stub.InitServer.return_value = mock_init_response
    mock_stub.ProcessSignal.return_value = mock_signal_response

    # Create a client with the mock stub
    client = BridgeClient("localhost", 50051)
    client.stub = mock_stub

    # Test initialize_server
    response = await client.initialize_server()
    assert response is not None
    assert response.success is True
    assert response.message == "Initialized"

    # Test process_signal
    request = pb2.SignalRequest(
        signal_id=1,
        workflow_id=123,
        signal_type=pb2.RUN,
    )
    response = await client.process_signal(request)
    assert response is not None
    assert response.success is True
    assert response.message == "Processed"

    # Verify the stub was called correctly
    mock_stub.InitServer.assert_called_once()
    mock_stub.ProcessSignal.assert_called_once_with(request)

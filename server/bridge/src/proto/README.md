# gRPC Client for Portico Bridge

This directory contains the gRPC client code for the Portico Bridge service.
The code is generated from the proto files in the `server/proto` directory.

## Generating gRPC Code

To (re)generate the gRPC code, run from within the bridge directory:

```bash
python -m src.proto.build_proto
```

## Using the gRPC Client

The gRPC client is automatically initialized in the main.py file.
See `lib.py` for implementation details of the BridgeClient class.

## Development Notes

- The proto files are shared between the Rust engine and Python bridge
- The build script automatically converts proto definitions to Python
- Run the build script whenever the proto files change
- The client handles conversion between Python dicts and protobuf messages

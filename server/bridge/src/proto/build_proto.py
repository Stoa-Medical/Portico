#!/usr/bin/env python
"""
Build script to generate Python gRPC code from proto files.
Run this script to regenerate the gRPC code after changes to the proto files.
"""

import os
import sys
import subprocess
import re

# Get the absolute path to the project root
REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../../"))
PROTO_PATH = os.path.join(REPO_ROOT, "server/proto")
OUTPUT_PATH = os.path.join(REPO_ROOT, "server/bridge/src/proto")


def fix_imports():
    """Fix imports in generated protobuf files."""
    print("Fixing imports in generated files...")

    # Fix imports in *_pb2_grpc.py files
    for filename in os.listdir(OUTPUT_PATH):
        if filename.endswith("_pb2_grpc.py"):
            file_path = os.path.join(OUTPUT_PATH, filename)

            # Read the file
            with open(file_path, "r") as file:
                content = file.read()

            # Replace imports
            content = re.sub(
                r"import (\w+)_pb2 as", r"from src.proto import \1_pb2 as", content
            )

            # Write back to file
            with open(file_path, "w") as file:
                file.write(content)

            print(f"Fixed imports in {filename}")


def generate_proto_code():
    """Generate Python code from proto files."""
    print(f"Generating Python code from proto files in {PROTO_PATH}...")

    # Ensure the output directory exists
    os.makedirs(OUTPUT_PATH, exist_ok=True)

    proto_files = [f for f in os.listdir(PROTO_PATH) if f.endswith(".proto")]

    if not proto_files:
        print("No proto files found!")
        return False

    for proto_file in proto_files:
        proto_file_path = os.path.join(PROTO_PATH, proto_file)
        print(f"Processing {proto_file}...")

        try:
            cmd = [
                "python",
                "-m",
                "grpc_tools.protoc",
                f"--proto_path={PROTO_PATH}",
                f"--python_out={OUTPUT_PATH}",
                f"--grpc_python_out={OUTPUT_PATH}",
                proto_file_path,
            ]
            subprocess.check_call(cmd)
            print(f"Successfully generated code for {proto_file}")
        except subprocess.CalledProcessError as e:
            print(f"Error generating code for {proto_file}: {e}")
            return False

    # Fix imports in generated files
    fix_imports()

    print("All proto files processed successfully!")
    return True


if __name__ == "__main__":
    if not generate_proto_code():
        sys.exit(1)
    sys.exit(0)

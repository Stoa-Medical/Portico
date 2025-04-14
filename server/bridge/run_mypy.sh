#!/bin/bash
# This script runs mypy with the correct configuration for the bridge module

cd $(dirname "$0")  # Change to the script's directory
mypy --config-file=mypy.ini src
exit $?

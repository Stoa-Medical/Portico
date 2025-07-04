#!/bin/bash
set -e

# Color definitions
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Function to handle cleanup
cleanup() {
  echo -e "${YELLOW}Stopping services...${NC}"
  docker compose down
  echo -e "${GREEN}Services stopped${NC}"
}

# Trap Ctrl+C
trap cleanup INT

# Check for Docker
if ! command -v docker &> /dev/null; then
  echo -e "${RED}Error: Docker is not installed or not in PATH${NC}"
  exit 1
fi

# Check if Docker Compose is available
if ! docker compose version &> /dev/null; then
  echo -e "${RED}Error: Docker Compose is not available${NC}"
  exit 1
fi

echo -e "${GREEN}Starting Portico services...${NC}"

# Build and start the services
docker compose up --build -d

# Check if services started successfully
if [ $? -ne 0 ]; then
  echo -e "${RED}Failed to start services${NC}"
  exit 1
fi

echo -e "${GREEN}Services started successfully!${NC}"
echo -e "View logs with: ${YELLOW}docker compose logs -f${NC}"
echo -e "Stop services with: ${YELLOW}docker compose down${NC} or press Ctrl+C if this script is running"
echo -e "${GREEN}Portico is now running!${NC}"

# Keep the script running to allow easy termination with Ctrl+C
echo -e "${YELLOW}Press Ctrl+C to stop all services...${NC}"
wait

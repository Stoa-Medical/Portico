# Glossary

## Core Concepts

### Agent
Orchestrator that plans and validates Workflows. Agents can initiate Workflow runs. Each Agent:
- Has capabilities and policies that constrain what workflows they can create
- Plans workflow specifications based on objectives
- Validates workflow specs against security and policy constraints
- Can trigger execution of approved workflows

### Signal
An event that triggers actions within the system. Three types:
- **run**: Triggers a Workflow execution with specific data
- **sync**: Forces the Engine to refresh its state from the database
- **fyi**: Logs data with timestamp without triggering execution

### Workflow
Executable unit that contains Steps and owns execution queue. Each Workflow:
- Contains one or more Steps executed in sequence
- Processes Signals through its own FIFO queue
- Creates RuntimeSessions to track execution
- Can be created by Agents or manually

### Step
A unit of action within a Workflow. Two types:
- **Deterministic**: Python code execution
- **Non-deterministic**: LLM prompt execution

Each Step expects JSON input and returns `Result<Value, Error>`.

### RuntimeSession
The execution record created when a Workflow processes a Signal. Tracks:
- Execution timestamps and duration
- Success/failure status of each Step
- Input/output data
- Error details if applicable
- Optional link to the initiating Agent

## Architecture Components

### Engine
The Rust-based runtime service that:
- Executes Workflows in a thread-pool
- Maintains in-memory Workflow state
- Exposes gRPC API for workflow planning and execution
- Persists results to PostgreSQL

### Bridge
Python service that acts as middleware between Supabase and Engine:
- Subscribes to Supabase Realtime events
- Converts database events to gRPC calls
- Handles retry logic and error propagation

### Supabase
Managed PostgreSQL platform providing:
- Database storage (source of truth)
- Authentication services
- Realtime event streaming
- REST API (PostgREST)

## Technical Terms

### FIFO Queue
First-In-First-Out queue maintained per Workflow to ensure ordered processing of Signals.

### gRPC
Google's Remote Procedure Call framework used for Engine ↔ Bridge communication.

### IPC
Inter-Process Communication used between Tauri frontend and backend.

### PostgREST
Automatic REST API generated from PostgreSQL schema by Supabase.

### Realtime
Supabase's WebSocket-based system for streaming database changes.

### Thread Pool
Shared worker threads in Engine that process Workflow queues concurrently.

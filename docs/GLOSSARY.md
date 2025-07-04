# Glossary

## Core Concepts

### Agent
An automation unit that processes data through a series of Steps. Each Agent:
- Has a unique identifier and descriptive goal
- Contains one or more Steps executed in sequence
- Processes Signals through its own FIFO queue
- Creates RuntimeSessions to track execution

### Signal
An event that triggers actions within the system. Three types:
- **run**: Triggers an Agent execution with specific data
- **sync**: Forces the Engine to refresh its state from the database
- **fyi**: Logs data with timestamp without triggering execution

### Step
A unit of action within an Agent. Two types:
- **Deterministic**: Python code execution
- **Non-deterministic**: LLM prompt execution

Each Step expects JSON input and returns `Result<Value, Error>`.

### RuntimeSession
The execution record created when an Agent processes a Signal. Tracks:
- Execution timestamps and duration
- Success/failure status of each Step
- Input/output data
- Error details if applicable

## Architecture Components

### Engine
The Rust-based runtime service that:
- Executes Agents in a thread-pool
- Maintains in-memory Agent state
- Exposes gRPC API for communication
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
First-In-First-Out queue maintained per Agent to ensure ordered processing of Signals.

### gRPC
Google's Remote Procedure Call framework used for Engine ↔ Bridge communication.

### IPC
Inter-Process Communication used between Tauri frontend and backend.

### PostgREST
Automatic REST API generated from PostgreSQL schema by Supabase.

### Realtime
Supabase's WebSocket-based system for streaming database changes.

### Thread Pool
Shared worker threads in Engine that process Agent queues concurrently.

# Portico Server

The Portico server implements a microservices architecture with agent-driven workflow composition and execution.

## Architecture Overview

```
Frontend ↔ Database ← Bridge → Engine
              ↕
           Agent System
```

### Core Services

1. **PostgreSQL Database** - Core data storage with realtime subscriptions
2. **Python Bridge** (`/bridge`) - Connects database events to engine via gRPC
3. **Rust Engine** (`/engine`) - Processes workflows and manages agent composition
4. **Shared Library** (`/shared`) - Database models and agent system

### Agent-Workflow Architecture

Portico implements a clear separation where:
- **Agents compose workflows** (intelligence) - analyze objectives, create specifications, apply policies
- **Workflows execute tasks** (efficiency) - process steps, manage runtime state, report results

#### Agent System Components

**Agent Capabilities & Policies:**
```rust
pub struct Agent {
    pub capabilities: AgentCapabilities,  // Tools, models, limits
    pub policy: AgentPolicy,             // Security, constraints, patterns
}
```

**Workflow Composition Flow:**
1. Agent receives objective and context
2. WorkflowPlanner creates workflow specification
3. WorkflowValidator validates against capabilities and policies
4. Agent triggers execution via Signal
5. WorkflowEngine executes steps
6. GarbageCollector cleans up ephemeral workflows

## Quick Start

### Start All Services
```bash
cd server
./start.sh
```

This launches:
- PostgreSQL database in Docker
- Python bridge service
- Rust engine service
- All necessary dependencies

### Individual Service Management

**Database:**
```bash
docker compose up postgres -d
docker compose logs -f postgres
```

**Bridge Service:**
```bash
cd bridge
./install_deps.sh
source .venv/bin/activate
python -m src.main
```

**Engine Service:**
```bash
cd engine
cargo run
```

## Agent System Usage

### Creating and Managing Agents

```rust
use shared::models::{Agent, AgentManager, AgentCapabilities, AgentPolicy};

// Create agent manager
let mut manager = AgentManager::new(database_pool).await;

// Start automatic cleanup
manager.start_auto_gc(24).await?; // Every 24 hours

// Plan and execute workflow
let result = manager.compose_execute_and_monitor(
    agent_id: 1,
    objective: "Analyze customer data and generate insights",
    context: Some(json!({"data_source": "customers.csv"})),
    constraints: Some(json!({"max_steps": 10})),
    is_ephemeral: true,
    auto_execute: false,
    enforce_strict_validation: true
).await?;

if result.success {
    println!("Workflow completed: {}", result.message);
}
```

### Agent Composition via gRPC

The engine exposes a `PlanWorkflow` RPC endpoint:

```protobuf
rpc PlanWorkflow(PlanWorkflowRequest) returns (PlanWorkflowResponse);
```

Example usage from bridge:
```python
response = await bridge_client.plan_workflow(
    agent_id=1,
    objective="Process quarterly sales data",
    context={"quarter": "Q1", "year": 2024},
    is_ephemeral=False
)
```

### Validation and Security

The system includes comprehensive validation:
- **Structure validation** - workflow format and required fields
- **Capability validation** - agent can use all required tools/models
- **Policy compliance** - respects step limits and allowed patterns
- **Security validation** - scans for dangerous operations
- **Performance validation** - warns about complex workflows

## Database Schema

### Core Tables
- `agents` - Agent definitions with capabilities and policies
- `workflows` - Executable workflow specifications (replaces old agents table)
- `steps` - Individual workflow steps
- `runtime_sessions` - Workflow execution tracking
- `signals` - Workflow execution triggers

### Key Relationships
```sql
agents (1) ← (many) workflows     -- Agents create workflows
workflows (1) ← (many) steps      -- Workflows contain steps
workflows (1) ← (many) signals    -- Signals trigger workflows
workflows (1) ← (many) runtime_sessions -- Execution tracking
```

## Development

### Database Operations
```bash
# Reset database schema
./reset_db.sh

# Connect to database
psql $DATABASE_URL

# View logs
docker compose logs -f postgres
```

### Testing
```bash
# Test shared library (includes agent tests)
cd shared && cargo test

# Test engine
cd engine && cargo test

# Test bridge
cd bridge && source .venv/bin/activate && pytest

# Run agent integration tests
cd shared && cargo test agent_integration_tests --ignored
```

### Environment Variables
Copy example files and configure:
```bash
cp engine/.env-example engine/.env
cp bridge/.env-example bridge/.env
```

Required variables:
- `DATABASE_URL` - PostgreSQL connection
- `ENGINE_HOST/PORT` - gRPC service location
- Service-specific configuration

## Agent System Features

### Ephemeral Workflows
- Temporary workflows for experimentation and one-time tasks
- Automatic garbage collection based on configurable policies
- Ideal for data exploration and quick analysis

### Garbage Collection
- **Automatic cleanup** of completed ephemeral workflows
- **Policy-based retention** for failed workflows and old sessions
- **Orphaned resource cleanup** for steps without parent workflows
- **Configurable schedules** and retention periods

### Validation System
- **Multi-layer validation** including structure, capabilities, policies, and security
- **Approval workflows** for complex or risky operations
- **Performance warnings** for resource-intensive workflows
- **Security scanning** for dangerous code patterns

### Agent Policies
- **Rate limiting** - workflows per hour restrictions
- **Tool restrictions** - blacklist dangerous or inappropriate tools
- **Pattern matching** - restrict workflow types to approved patterns
- **Security constraints** - custom security rules and approval requirements

## Architecture Benefits

### Scalability
- Services can be deployed and scaled independently
- Agent composition and workflow execution are decoupled
- Database handles concurrent access and realtime updates

### Security
- Multiple validation layers prevent dangerous operations
- Agent policies enforce organizational constraints
- Audit trail of all composition and execution activities

### Maintainability
- Clear separation between composition intelligence and execution efficiency
- Well-defined interfaces between services
- Comprehensive testing at unit and integration levels

## Monitoring and Observability

### Agent Statistics
```rust
let stats = manager.get_comprehensive_agent_stats(agent_id).await?;
println!("Workflows created: {}", stats.basic_stats.workflows_created);
println!("Cleanup candidates: {}", stats.cleanup_candidates.total_items());
```

### Garbage Collection Monitoring
```rust
let gc_stats = manager.run_manual_gc().await?;
println!("Cleaned up {} ephemeral workflows", gc_stats.ephemeral_workflows_cleaned);
```

### Service Health
- Bridge service logs composition requests and database events
- Engine service logs gRPC requests and workflow execution
- Database connection pooling and query performance monitoring

For detailed architectural documentation, see `/ai_docs/agent_architecture.md`.

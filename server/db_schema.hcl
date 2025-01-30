# Using the Atlas database build tool to manage schema
#   docs: https://atlasgo.io/

/*
NOTE: Before applying: `psql "postgresql://postgres:postgres@localhost:54322/postgres?sslmode=disable" -c "CREATE DATABASE server;"`
  To apply: 
  
atlas schema apply \
--url "postgresql://postgres:postgres@localhost:54322/postgres?&sslmode=disable" \
--to "file://db_schema.hcl" \
--schema "public"
*/

schema "public" {}

// Define enums first
enum "agent_state" {
  schema = schema.public
  values = [
    "inactive",
    "waiting",
    "running",
    "unstable",
    "stopping"
  ]
}

enum "agent_type" {
  schema = schema.public
  values = [
    "actor",
    "reactor"
  ]
}

enum "step_action" {
  schema = schema.public
  values = [
    "python",
    "prompt"
  ]
}

// public tables
table "agents" {
  schema = schema.public
  column "id" {
    type = varchar(36)  // UUID
    null = false
  }
  
  primary_key {
    columns = [column.id]
  }

  column "description" {
    type = text
    null = false
  }
  column "agent_state" {
    type = enum.agent_state
    null = false
  }
  column "agent_type" {
    type = enum.agent_type
    null = false
  }
  column "accepted_err_rate" {
    type = float
    null = false
  }
}

table "steps" {
  schema = schema.public
  column "id" {
    type = int
    null = false
    identity {
      generated = ALWAYS
    }
  }

  primary_key {
    columns = [column.id]
  }

  column "name" {
    type = varchar(255)
    null = false
  }
  column "action_type" {
    type = enum.step_action
    null = false
  }
  column "action_content" {
    type = text
    null = false
  }
  column "run_count" {
    type = bigint
    null = false
    default = 0
  }
  column "success_count" {
    type = bigint
    null = false
    default = 0
  }
  column "agent_id" {
    type = varchar(36)
    null = false
  }

  foreign_key "step_agent_fk" {
    columns = [column.agent_id]
    ref_columns = [table.agents.column.id]
    on_delete = CASCADE
  }
}

table "runtime_sessions" {
  schema = schema.public
  column "id" {
    type = int
    null = false
    identity {
      generated = ALWAYS
    }
  }

  primary_key {
    columns = [column.id]
  }

  column "current_step" {
    type = int
    null = false
  }
  column "saved_result" {
    type = json
    null = true
  }
  column "completed" {
    type = bool
    null = false
    default = false
  }
  column "curr_idx" {
    type = int
    null = false
    default = 0
  }
  column "agent_id" {
    type = varchar(36)
    null = false
  }

  foreign_key "rts_agent_fk" {
    columns = [column.agent_id]
    ref_columns = [table.agents.column.id]
    on_delete = CASCADE
  }
}
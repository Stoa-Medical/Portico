# Using the Atlas database build tool to manage schema
#   docs: https://atlasgo.io/

/*
NOTE: Before applying: `psql "postgresql://postgres:postgres@localhost:54322/postgres?sslmode=disable" -c "CREATE DATABASE server;"`
  To apply: 
  
atlas schema apply \
--url "postgresql://postgres:postgres@localhost:54322/postgres?&sslmode=disable" \
--to "file://db_schema.hcl" \
--schema "public"

NOTE: the Atlas HCL syntax is custom and doesn't support everything yet (e.g. `mixin`, `domain`, etc.)
  Docs: https://atlasgo.io/atlas-schema/hcl

To reset database state -- generation for the YOLO SQL below (run in `psql`):
```sql
-- Generate drop statements for all tables in public schema
SELECT 'DROP TABLE IF EXISTS "' || tablename || '" CASCADE;'
FROM pg_tables 
WHERE schemaname = 'public' 
  AND tablename NOT LIKE 'supabase_%';
```
... then copy that code and drop the tables
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

enum "job_status" {
  schema = schema.public
  values = [
    "waiting",
    "in_progress",
    "completed",
    "cancelled"
  ]
}

enum "message_type" {
  schema = schema.public
  values = [
    "agent_job"
  ]
}

// public tables
table "agents" {
  schema = schema.public

  // Incrementing int ID
  column "id" {
    type = bigint
    null = false
    identity {
      generated = ALWAYS
    }
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

  // Incrementing int ID
  column "id" {
    type = bigint
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
    type = bigint
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
    type = bigint
    null = false
    identity {
      generated = ALWAYS
    }
  }
  primary_key {
    columns = [column.id]
  }

  column "current_step" {
    type = bigint
    null = false
  }
  column "source_data" {
    type = json
    null = false
  }
  column "current_result" {
    type = json
    null = true
  }
  column "completed" {
    type = bool
    null = false
    default = false
  }
  column "last_idx" {
    type = int
    null = false
    default = 0
  }
  column "agent_id" {
    type = bigint
    null = false
  }

  foreign_key "rts_agent_fk" {
    columns = [column.agent_id]
    ref_columns = [table.agents.column.id]
    on_delete = CASCADE
  }

}

table "user_jobs" {
  # TODO: Update this
  schema = schema.public

  column "id" {
    type = bigint
    null = false
    identity {
      generated = ALWAYS
    }
  }
  primary_key {
    columns = [column.id]
  }

  column "description" {
    type = varchar(255)
    null = false
  }

  column "requested_agent_id" {
    type = bigint
    null = false
  }

  column "starting_data" {
    type = json
    null = false
  }

  column "user_id" {
    type = uuid
    null = false
  }

  column "job_status" {
    type enum.job_status
    null = false
  }
}

table "messages" {
  column "message_type" {

  }
}
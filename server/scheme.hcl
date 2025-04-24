# Using the Atlas database build tool to manage schema
#     docs: https://atlasgo.io/

/*
NOTE: Before applying: `psql "postgresql://postgres:postgres@localhost:54322/postgres?sslmode=disable" -c "CREATE DATABASE server;"`
    To apply:

atlas schema apply \
  --url "postgresql://postgres:postgres@localhost:54322/postgres?search_path=public&sslmode=disable" \
  --to "file://scheme.hcl"

NOTE: the Atlas HCL syntax is custom and doesn't support everything yet (e.g. `mixin`, `domain`, etc.)
    Docs: https:atlasgo.io/atlas-schema/hcl

To seed some data into the database, run the `supabase/seed.sql` file with: `psql postgresql://postgres:postgres@localhost:54322/postgres -f seed.sql`

To reset database state -- you can use the Supabase CLI (`supabase db reset`)
  OR run generation for the YOLO SQL below (run in `psql`):
```
psql -h localhost -p 54322 -U postgres -d postgres
```

```sql
-- Generate drop statements for all tables in public schema
SELECT 'DROP TABLE IF EXISTS "' || tablename || '" CASCADE;'
FROM pg_tables
WHERE schemaname = 'public'
    AND tablename NOT LIKE 'supabase_%';

--- ... then copy that code and drop the tables

--- If you also need to reset types:
-- Generate drop statements for all enum types in the database
SELECT 'DROP TYPE IF EXISTS "' || t.typname || '" CASCADE;'
FROM pg_type t
JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace
WHERE n.nspname = 'public'
    AND t.typtype = 'e'
    AND t.typname NOT LIKE 'supabase_%';

--- ... then do it again!
```
*/


schema "public" {}

# ============ Tables ============
# NOTE: Naming convention is `{model_name}s`, note the plural

table "signals" {
    # === General ===
    schema = schema.public

    # === Ids ===
    column "id" {
        type = sql("bigint")
        null = false
        identity {
            generated = "ALWAYS"
        }
    }

    column "global_uuid" {
        type = sql("uuid")
        null = false
        default = sql("gen_random_uuid()")
    }

    primary_key {
        columns = [
            column.id
        ]
    }

    # === Relationships ===
    column "agent_id" {
        type = int
        null = true
    }

    foreign_key "signal_agent_fk" {
        columns = [
            column.agent_id
        ]
        ref_columns = [
            table.agents.column.id
        ]
    }

    column "rts_id" {
        type = sql("bigint")
        null = true
    }

    foreign_key "signal_rts_fk" {
        columns = [
            column.rts_id
        ]
        ref_columns = [
            table.runtime_sessions.column.id
        ]
    }

    # === Timestamps ===
    column "created_at" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    column "updated_at" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    # === Custom (table-specific) ===
    column "user_requested_uuid" {
        type = sql("uuid")
        null = false
    }

    column "signal_type" {
        type = enum.signal_type
        null = false
        default = "fyi"
    }

    column "initial_data" {
        type = sql("json")
        null = true
    }

    column "response_data" {
        type = sql("json")
        null = true
    }

    column "error_message" {
        type = sql("text")
        null = true
    }
}

table "agents" {
    # === General ===
    schema = schema.public

    # === Ids ===
    column "id" {
        type = int
        null = false
        identity {
            generated = "ALWAYS"
        }
    }

    column "global_uuid" {
        type = sql("uuid")
        null = false
        default = sql("gen_random_uuid()")
    }

    primary_key {
        columns = [
            column.id
        ]
    }

    # === Relationships ===
    # An Agent can have many Steps (a Step points to an Agent)
    column "step_ids" {
        type = sql("int[]")
        null = true
        comment = "Array of step IDs in execution order"
    }

    # === Timestamps ===
    column "created_at" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    column "updated_at" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    # === Custom (table-specific) ===
    column "name" {
        type = sql("varchar(255)")
        null = true
    }
    column "type" {
        type = sql("varchar(255)")
        null = true
    }
    column "description" {
        type = sql("text")
        null = true
    }
    column "agent_state" {
        type = enum.agent_state
        null = false
    }
    column "agent_name" {
        type = sql("text")
        null = true
    }
    column "agent_type" {
        type = sql("text")
        null = true
    }
}

table "steps" {
    # === General ===
    schema = schema.public

    # === Ids ===
    column "id" {
        type = int
        null = false
        identity {
            generated = "ALWAYS"
        }
    }

    column "global_uuid" {
        type = sql("uuid")
        null = false
        default = sql("gen_random_uuid()")
    }

    primary_key {
        columns = [
            column.id
        ]
    }

    # === Relationships ===
    # A Step is defined within an Agent, and has a unique sequence
    # An Agent runs steps in ascending order of sequence numbers
    column "agent_id" {
        type = int
        null = false
    }
    foreign_key "step_agent_fk" {
        columns = [
            column.agent_id
        ]
        ref_columns = [
            table.agents.column.id
        ]
    }

    # === Timestamps ===
    column "created_at" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    column "updated_at" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    # === Custom (table-specific) ===
    column "name" {
        type = sql("varchar(255)")
        null = true
    }
    column "description" {
        type = sql("text")
        null = true
    }
    column "step_type" {
        type = enum.step_type
        null = false
    }
    column "step_content" {
        type = sql("text")
        null = false
    }
}

table "runtime_sessions" {
    # === General ===
    schema = schema.public

    # === Ids ===
    column "id" {
        type = sql("bigint")
        null = false
        identity {
            generated = "ALWAYS"
        }
    }

    column "global_uuid" {
        type = sql("uuid")
        null = false
        default = sql("gen_random_uuid()")
    }

    primary_key {
        columns = [
            column.id
        ]
    }

    # === Relationships ===
    column "requested_by_agent_id" {
        type = int
        null = false
    }
    foreign_key "runtime_session_agent_fk" {
        columns = [
            column.requested_by_agent_id
        ]
        ref_columns = [
            table.agents.column.id
        ]
    }

    # === Timestamps ===
    column "created_at" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    column "updated_at" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    # === Custom (table-specific) ===
    column "rts_status" {
        type = enum.running_status
        null = false
    }
    column "initial_data" {
        type = sql("json")
        null = false
    }
    column "latest_step_idx" {
        type = int
        null = false
    }
    column "latest_result" {
        type = sql("json")
        null = true
    }
    column "step_execution_times" {
        type = sql("numeric(20,6)[]")
        null = true
        comment = "Array of execution times in seconds with microsecond precision"
    }
    column "step_ids" {
        type = sql("int[]")
        null = true
        comment = "Array of step IDs that were executed in this session"
    }
    column "total_execution_time" {
        type = sql("numeric(20,6)")
        null = true
        comment = "Total execution time in seconds with microsecond precision"
    }
}


# ============ enum ============

enum "signal_type" {
    schema = schema.public
    values = [
        "command",
        "sync",
        "fyi"
    ]
}

enum "agent_state" {
    schema = schema.public
    values = [
        "inactive",
        "stable",
        "unstable"
    ]
}

enum "step_type" {
    schema = schema.public
    values = [
        "python",
        "prompt"
    ]
}

# NOTE: This is shared with Mission + RuntimeSession
enum "running_status" {
    schema = schema.public
    values = [
        "waiting",  # This means it is on the queue (not started)
        "running",  # This means it is actively being worked on (in the thread)
        "completed",  # This means it was seen-through to completion (even if resulting data is error, workflow completed)
        "cancelled"   # This means it was intentionally cancelled (e.g. workflow error)
    ]
}

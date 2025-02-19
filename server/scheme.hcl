# Using the Atlas database build tool to manage schema
#     docs: https:#atlasgo.io/

/*
NOTE: Before applying: `psql "postgresql:#postgres:postgres@localhost:54322/postgres?sslmode=disable" -c "CREATE DATABASE server;"`
    To apply: 
    
atlas schema apply \
  --url "postgres://postgres:postgres@localhost:54322/postgres?search_path=public&sslmode=disable" \
  --to "file://scheme.hcl"

NOTE: the Atlas HCL syntax is custom and doesn't support everything yet (e.g. `mixin`, `domain`, etc.)
    Docs: https:atlasgo.io/atlas-schema/hcl


To reset database state -- generation for the YOLO SQL below (run in `psql`):
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
```
*/


# """
# TODO: Pin-down the schemas for:
# - [x] Signal
# - [ ] Schedule
# - [ ] Mission
# - [ ] Agent
# - [ ] Step
# - [ ] RuntimeSession

# - [ ] Try running this against supabase postgres schema
# - [ ] Define a `DatabaseItem` trait in lib.rs (like the good ol' days!)

# - [ ] Channel
# """

schema "public" {}

# ============ Tables ============
# NOTE: Naming convention is `{model_name}s`, note the plural

table "signals" {
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

    # === Timestamps ===
    column "created_timestamp" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    column "last_updated_timestamp" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    # === Provenance (auditing) ===
    column "audit_log" {
        type = sql("json")
        null = false
    }

    # === Custom (table-specific) ===
    column "signal_type" {
        type = enum.signal_type
        null = false
    }

    column "signal_status" {
        type = enum.signal_status
        null = false
    }

    column "initial_data" {
        type = sql("json")
        null = true
    }

    column "response" {
        type = sql("json")
        null = true
    }

    column "mission_id" {
        type = int
        null = false
    }
}


table "missions" {
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

    # === Timestamps ===
    column "created_timestamp" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    column "last_updated_timestamp" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    # === Provenance (auditing) ===
    column "audit_log" {
        type = sql("json")
        null = false
    }

    # === Custom (table-specific) ===
    column "mission_status" {
        type = enum.mission_status
        null = false
    }

    # NOTE: Create default users if automated in backend (e.g. "PORTICO_AGENT" or something)
    column "requested_user_id" {
        type = sql("uuid")
        null = false
    }

    column "description" {
        type = sql("varchar(255)")
        null = false
    }

    column "requested_agent_ids" {
        type = sql("integer[]")  # Represents in-order list of Agents (order matters for execution)
        null = false
    }

    column "initial_data" {
        type = sql("json")
        null = false
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

    # === Timestamps ===
    column "created_timestamp" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    column "last_updated_timestamp" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    # === Provenance (auditing) ===
    column "audit_log" {
        type = sql("json")
        null = false
    }

    # === Custom (table-specific) ===
    column "description" {
        type = sql("text")
        null = false
    }
    column "agent_state" {
        type = enum.agent_state
        null = false
    }
    column "accepted_err_rate" {
        type = float
        null = false
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

    # === Timestamps ===
    column "created_timestamp" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    column "last_updated_timestamp" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    # === Provenance (auditing) ===
    column "audit_log" {
        type = sql("json")
        null = false
    }

    # === Custom (table-specific) ===
    column "description" {
        type = sql("varchar(255)")
        null = false
    }
    column "action_type" {
        type = enum.step_action
        null = false
    }
    column "action_content" {
        type = sql("text")
        null = false
    }
    column "run_count" {
        type = int
        null = false
        default = 0
    }
    column "success_count" {
        type = int
        null = false
        default = 0
    }
    column "agent_id" {
        type = int
        null = false
    }
}

table "runtime_sessions" {
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

    # === Timestamps ===
    column "created_timestamp" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    column "last_updated_timestamp" {
        type = sql("timestamptz")
        null = false
        default = sql("CURRENT_TIMESTAMP")
    }

    # === Provenance (auditing) ===
    column "audit_log" {
        type = sql("json")
        null = false
    }

    # === Custom (table-specific) ===
    column "completed" {
        type = bool
        null = false
        default = false
    }
    column "initial_data" {
        type = sql("json")
        null = false
    }
    column "most_recent_step_idx" {
        type = int
        null = false
    }
    column "most_recent_result" {
        type = sql("json")
        null = true
    }
    column "requested_by_agent_id" {
        type = int
        null = false
    }
}


# ============ enum ============ 

enum "signal_type" {
    schema = schema.public

    # NOTE: be intentional about naming. Should be in past-tense
    #   Each one of these needs to be handled explicitly in the code
    #   As default convention: try `{struct-name}_{what-happened}`
    values = [
        "mission_user-requested",
        "mission_agent-requested",
        "mission_schedule-requested",
        "mission_channel-requested",
        "runtime-session_completed",
        "agent_saved",  # emit once existing Agent changes are saved
        "step_saved"  # emit once existing Step is saved
    ]
}

enum "signal_status" {
    schema = schema.public

    values = [
        "in-progress",
        "completed",  # This means it was seen-through to completion (even if resulting data is error, workflow completed)
        "cancelled"   # This means it was intentionally cancelled (e.g. workflow error)
    ]
}


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

enum "step_action" {
    schema = schema.public
    values = [
        "python",
        "prompt"
    ]
}

enum "mission_status" {
    schema = schema.public
    values = [
        "waiting",
        "in_progress",
        "completed",
        "cancelled"
    ]
}

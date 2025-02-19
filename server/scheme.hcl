# Using the Atlas database build tool to manage schema
#     docs: https://atlasgo.io/

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
# - [x] Mission
# - [x] Agent
# - [x] Step
# - [x] RuntimeSession

# - [ ] Try running this against supabase postgres schema
# - [ ] Define a `DatabaseItem` trait in lib.rs (like the good ol' days!)

# - [ ] Schedule
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
    column "mission_id" {
        type = int
        null = false
    }

    foreign_key "signal_mission_fk" {
        columns = [
            column.mission_id
        ]
        ref_columns = [
            table.missions.column.id
        ]
    }
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
    # NOTE: Create default users if automated in backend (e.g. "PORTICO_AGENT" or something)
    # NOTE: Not sure if I can make an explicit foreign key here, so will have to trust developer to not mess up
    column "user_requested_id" {
        type = sql("uuid")
        null = false
    }

    # Keep this one Agent for simplicity. Expand later with another column if needed
    # Should still have a "primary" Agent regardless for decision making
    column "requested_agent_id" {
        type = int
        null = false
    }

    foreign_key "mission_agent_fk" {
        columns = [
            column.requested_agent_id
        ]
        ref_columns = [
            table.agents.column.id
        ]
    }

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
        type = enum.running_status
        null = false
    }


    column "description" {
        type = sql("varchar(255)")
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
    # An Agent is assigned to a Mission (a Mission points to an Agent)
    # An Agent can have many Steps (a Step points to an Agent)

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
    column "sequence_number" {
        type = int
        null = false
    }

    unique "agent_step_order" {
        columns = [
            column.agent_id,
            column.sequence_number
        ]
    }

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
}

table "runtime_sessions" {
    # === General ===
    schema = schema.public

    # === Ids ===
    # NOTE: This is bigint because there could be a lot of these. Everything else should be int (update this comment if not the case)
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
    column "runtime_session_status" {
        type = enum.running_status
        null = false
    }
    column "initial_data" {
        type = sql("json")
        null = false
    }
    column "most_recent_step_number" {
        type = int
        null = false
    }
    column "most_recent_result" {
        type = sql("json")
        null = true
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

# NOTE: This is shared with Mission + RuntimeSession
enum "running_status" {
    schema = schema.public
    values = [
        "waiting",
        "in_progress",
        "completed",
        "cancelled"
    ]
}

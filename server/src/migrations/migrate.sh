#!/bin/bash
DB_NAME="prod"
MIGRATIONS_DIR="./versions"

# Get current version
current_version=$(psql $DB_NAME -t -c "SELECT COALESCE(MAX(version), 0) FROM schema_migrations;")

# Apply pending migrations in order
for f in $MIGRATIONS_DIR/v*__*.sql; do
    version=$(echo $f | sed 's/.*V\([0-9]*\)__.*/\1/')
    if [ "$version" -gt "$current_version" ]; then
        echo "Applying migration $f..."
        psql $DB_NAME -f "$f"
    fi
done

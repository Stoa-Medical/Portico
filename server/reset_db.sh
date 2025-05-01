#!/bin/bash
supabase db reset
atlas schema apply \
  --url "postgresql://postgres:postgres@localhost:54322/postgres?search_path=public&sslmode=disable" \
  --to "file://scheme.hcl" \
  --auto-approve
psql postgresql://postgres:postgres@localhost:54322/postgres -f seed.sql

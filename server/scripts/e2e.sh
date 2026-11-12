#!/usr/bin/env bash
# End-to-end check: a signal for an agent runs that agent's steps and settles.
#
#   ./scripts/e2e.sh                         insert signals directly into Postgres
#   WEB_URL=http://localhost:3000 ./scripts/e2e.sh   create them through POST /api/signals
#
# Brings up the compose stack, seeds two agents, and asserts on the rows the
# engine writes. Exits non-zero on the first failed assertion.
set -euo pipefail
cd "$(dirname "$0")/.."

WEB_URL="${WEB_URL:-}"
WAIT_SECONDS="${WAIT_SECONDS:-60}"

psql() { docker compose exec -T postgres psql -U postgres -v ON_ERROR_STOP=1 -At "$@"; }

fail() { echo "FAIL: $1" >&2; exit 1; }

assert_eq() {
  [[ "$2" == "$3" ]] || fail "$1: expected '$3', got '$2'"
  echo "ok: $1 = $3"
}

docker compose up -d --build

for _ in $(seq 60); do
  [[ "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/health || true)" == "200" ]] && break
  sleep 2
done
assert_eq "engine GET /health" "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/health || true)" "200"

run_id="$(uuidgen | tr 'A-Z' 'a-z')"

seed_agent() {
  local name="$1" script="$2"
  psql -v name="$name" -v script="$script" <<'SQL'
WITH a AS (
  INSERT INTO agents (name) VALUES (:'name') RETURNING id
), s AS (
  INSERT INTO steps (agent_id, name, step_type, step_order, config)
  SELECT id, 'passthrough', 'transform'::step_type, 1, NULL::jsonb FROM a
  UNION ALL
  SELECT id, 'compute', 'python'::step_type, 2, jsonb_build_object('script', :'script') FROM a
)
SELECT id FROM a;
SQL
}

create_signal() {
  local agent_id="$1" key="$2" payload="$3"
  if [[ -n "$WEB_URL" ]]; then
    curl -sf -X POST "$WEB_URL/api/signals" -H 'content-type: application/json' \
      -d "{\"signalType\":\"run\",\"source\":\"e2e\",\"agentId\":\"$agent_id\",\"idempotencyKey\":\"$key\",\"payload\":$payload}" \
      | sed -E 's/.*"id":"([^"]+)".*/\1/'
  else
    psql -v agent="$agent_id" -v key="$key" -v payload="$payload" <<'SQL' | head -1
INSERT INTO signals (signal_type, source, agent_id, idempotency_key, payload)
VALUES ('run', 'e2e', :'agent', :'key', :'payload'::jsonb)
RETURNING id;
SQL
  fi
}

wait_settled() {
  local signal_id="$1" status=""
  for _ in $(seq "$WAIT_SECONDS"); do
    status="$(psql -c "SELECT status FROM signals WHERE id = '$signal_id'")"
    [[ "$status" == "completed" || "$status" == "failed" ]] && { echo "$status"; return; }
    sleep 1
  done
  echo "$status"
}

echo "--- a run that succeeds"
good_agent="$(seed_agent "e2e-good-$run_id" 'output_data = {"doubled": input_data["n"] * 2}')"
good_signal="$(create_signal "$good_agent" "e2e-good-$run_id" '{"n": 21}')"
assert_eq "signal status" "$(wait_settled "$good_signal")" "completed"
assert_eq "signal response_data" \
  "$(psql -c "SELECT response_data::text FROM signals WHERE id = '$good_signal'")" '{"doubled": 42}'
assert_eq "session status" \
  "$(psql -c "SELECT rs.status FROM runtime_sessions rs JOIN signals s ON s.runtime_session_id = rs.id WHERE s.id = '$good_signal'")" "completed"
assert_eq "session step statuses" \
  "$(psql -c "SELECT string_agg(r->>'name' || ':' || (r->>'status'), ',' ORDER BY ord) FROM runtime_sessions rs, jsonb_array_elements(rs.step_results) WITH ORDINALITY AS t(r, ord) WHERE rs.signal_id = '$good_signal'")" \
  "passthrough:completed,compute:completed"
assert_eq "session last step output" \
  "$(psql -c "SELECT (step_results->-1->'output')::text FROM runtime_sessions WHERE signal_id = '$good_signal'")" '{"doubled": 42}'
assert_eq "lease cleared" \
  "$(psql -c "SELECT (leased_at IS NULL AND lease_expires_at IS NULL)::text FROM signals WHERE id = '$good_signal'")" "true"

echo "--- a run whose Python step raises"
bad_agent="$(seed_agent "e2e-bad-$run_id" 'raise ValueError("boom")')"
bad_signal="$(create_signal "$bad_agent" "e2e-bad-$run_id" '{"n": 1}')"
assert_eq "signal status" "$(wait_settled "$bad_signal")" "failed"
assert_eq "signal error names the step and cause" \
  "$(psql -c "SELECT (error_message LIKE 'step ''compute'' failed:%ValueError: boom%')::text FROM signals WHERE id = '$bad_signal'")" "true"
assert_eq "session status" \
  "$(psql -c "SELECT status FROM runtime_sessions WHERE signal_id = '$bad_signal'")" "failed"
assert_eq "session step statuses" \
  "$(psql -c "SELECT string_agg(r->>'name' || ':' || (r->>'status'), ',' ORDER BY ord) FROM runtime_sessions rs, jsonb_array_elements(rs.step_results) WITH ORDINALITY AS t(r, ord) WHERE rs.signal_id = '$bad_signal'")" \
  "passthrough:completed,compute:failed"

echo "--- a signal left dispatched by a dead engine is rerun after restart"
orphan_signal="$(psql -v agent="$good_agent" -v key="e2e-orphan-$run_id" <<'SQL' | head -1
WITH s AS (
  INSERT INTO signals (signal_type, source, agent_id, idempotency_key, payload, status, leased_at, lease_expires_at)
  VALUES ('run', 'e2e', :'agent', :'key', '{"n": 5}', 'dispatched', now(), now() + interval '1 hour')
  RETURNING id, agent_id
), rs AS (
  INSERT INTO runtime_sessions (agent_id, signal_id, status) SELECT agent_id, id, 'running' FROM s
)
SELECT id FROM s;
SQL
)"
docker compose restart engine
assert_eq "signal status" "$(wait_settled "$orphan_signal")" "completed"
assert_eq "signal response_data" \
  "$(psql -c "SELECT response_data::text FROM signals WHERE id = '$orphan_signal'")" '{"doubled": 10}'
assert_eq "session statuses, oldest first" \
  "$(psql -c "SELECT string_agg(status::text, ',' ORDER BY created_at) FROM runtime_sessions WHERE signal_id = '$orphan_signal'")" \
  "cancelled,completed"

if [[ -n "$WEB_URL" ]]; then
  echo "--- a webhook routes to its integration's agent"
  secret="e2e-secret-$run_id"
  secret_hash="$(printf '%s' "$secret" | shasum -a 256 | cut -d' ' -f1)"
  hl7_agent="$(seed_agent "e2e-hl7-$run_id" 'output_data = {"has_msh": input_data["raw"].startswith("MSH")}')"
  seed_integration() {
    psql -v name="e2e-$1-$run_id" -v agent="$hl7_agent" -v hash="$secret_hash" -v status="$1" <<'SQL' | head -1
INSERT INTO integrations (name, connection_type, direction, agent_id, status, config)
VALUES (:'name', 'hl7v2-http', 'inbound', :'agent', :'status'::integration_status,
        jsonb_build_object('webhook_secret_sha256', :'hash'))
RETURNING id;
SQL
  }
  integration="$(seed_integration active)"
  inactive_integration="$(seed_integration inactive)"
  hl7_message=$'MSH|^~\\&|LAB|HOSP|PORTICO|STOA|20260101||ORU^R01|CTRL-'"$run_id"$'|P|2.5\rPID|1||12345'

  post_hl7() {
    curl -s -o "$3" -w '%{http_code}' -X POST "$WEB_URL/api/webhooks/hl7/$1" \
      -H "authorization: Bearer $2" -H 'content-type: text/plain' --data-binary "$hl7_message"
  }
  body="$(mktemp)"

  assert_eq "POST with the right secret" "$(post_hl7 "$integration" "$secret" "$body")" "200"
  hl7_signal="$(sed -E 's/.*"signalId":"([^"]+)".*/\1/' "$body")"
  assert_eq "signal carries the integration's agent" \
    "$(psql -c "SELECT agent_id FROM signals WHERE id = '$hl7_signal'")" "$hl7_agent"
  assert_eq "signal status" "$(wait_settled "$hl7_signal")" "completed"
  assert_eq "signal response_data" \
    "$(psql -c "SELECT response_data::text FROM signals WHERE id = '$hl7_signal'")" '{"has_msh": true}'
  assert_eq "integration last_seen_at set" \
    "$(psql -c "SELECT (last_seen_at IS NOT NULL)::text FROM integrations WHERE id = '$integration'")" "true"

  assert_eq "retry of the same control id" "$(post_hl7 "$integration" "$secret" "$body")" "200"
  assert_eq "retry returns the first signal" "$(sed -E 's/.*"signalId":"([^"]+)".*/\1/' "$body")" "$hl7_signal"
  assert_eq "retry is flagged duplicate" "$(grep -o '"duplicate":[a-z]*' "$body")" '"duplicate":true'
  assert_eq "signals for the integration" \
    "$(psql -c "SELECT count(*) FROM signals WHERE idempotency_key LIKE '$integration:%'")" "1"

  assert_eq "POST with a wrong secret" "$(post_hl7 "$integration" "wrong" "$body")" "401"
  assert_eq "POST to an unknown integration" "$(post_hl7 "$(uuidgen | tr 'A-Z' 'a-z')" "$secret" "$body")" "404"
  assert_eq "POST to an inactive integration" "$(post_hl7 "$inactive_integration" "$secret" "$body")" "409"
  assert_eq "rejected requests created no signals" \
    "$(psql -c "SELECT count(*) FROM signals WHERE agent_id = '$hl7_agent'")" "1"
  rm -f "$body"
fi

echo "--- the session query the agent page runs returns the run"
assert_eq "sessions visible for the agent" \
  "$(psql -c "SELECT count(*) FROM runtime_sessions WHERE agent_id = '$good_agent' AND status = 'completed'")" "2"

echo "PASS"

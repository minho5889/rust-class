#!/usr/bin/env bash
# goldeneye telemetry v1 — append one envelope event (JSONL) to the local raw zone.
# Usage: telemetry.sh <event_type> [actor]   (optional hook JSON on stdin -> payload)
# Envelope schema: datalake/schema/envelope.v1.json
# Hard rule: never put secrets in payloads. Hooks must never fail the tool call.
set -uo pipefail

EVENT_TYPE="${1:-unknown}"
ACTOR="${2:-claude-main}"
ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
DIR="$ROOT/datalake/raw-local/dt=$(date -u +%F)"
mkdir -p "$DIR" || exit 0

STDIN_JSON="$(cat 2>/dev/null || true)"
PAYLOAD='{}'
if [ -n "$STDIN_JSON" ] && command -v jq >/dev/null 2>&1; then
  # Drop bulky duplicated conversation content; the transcript is the record of
  # what was said — the lake records that/when it happened.
  PAYLOAD="$(printf '%s' "$STDIN_JSON" | jq -c 'del(.last_assistant_message)' 2>/dev/null || echo '{}')"
fi

TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
EVENT_ID="$(date -u +%s%N)-$$-$RANDOM"
SESSION_ID="${CLAUDE_SESSION_ID:-unknown}"
SPEC_ID="${GOLDENEYE_SPEC_ID:-null}"
[ "$SPEC_ID" != "null" ] && SPEC_ID="\"$SPEC_ID\""

printf '{"event_id":"%s","ts":"%s","session_id":"%s","spec_id":%s,"actor":"%s","event_type":"%s","schema_version":1,"payload":%s}\n' \
  "$EVENT_ID" "$TS" "$SESSION_ID" "$SPEC_ID" "$ACTOR" "$EVENT_TYPE" "$PAYLOAD" \
  >> "$DIR/events.jsonl" 2>/dev/null

exit 0

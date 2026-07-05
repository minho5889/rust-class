#!/usr/bin/env bash
# goldeneye lake scan — standing queries over raw-local (jq; DuckDB later for traces).
# Rule: every query names the document it feeds. Run from repo root or anywhere.
set -uo pipefail
ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel 2>/dev/null || echo /home/user/rust-class)"
EVENTS=("$ROOT"/datalake/raw-local/dt=*/events.jsonl)

section() { printf '\n== %s\n   feeds: %s\n' "$1" "$2"; }

section "Event inventory" "sanity"
jq -s 'group_by(.event_type) | map({(.[0].event_type): length}) | add' "${EVENTS[@]}"

section "Gate funnel per spec (notified -> approved)" "MEMORY.md pipeline tuning"
jq -s '[.[] | select(.event_type | startswith("gate."))]
  | group_by(.spec_id)
  | map({spec: .[0].spec_id,
         notified: [.[] | select(.event_type=="gate.notified")] | length,
         approved: [.[] | select(.event_type=="gate.approved")] | length})' "${EVENTS[@]}"

section "Doc-write churn per spec doc (revisions proxy)" "MEMORY.md pipeline tuning"
jq -s '[.[] | select(.event_type=="spec.doc_written")]
  | group_by(.payload.file) | map({(.[0].payload.file): length}) | add' "${EVENTS[@]}"

section "Research findings by volatility" "research/ review queue"
jq -s '[.[] | select(.event_type=="research.finding")]
  | group_by(.payload.volatility) | map({(.[0].payload.volatility): length}) | add' "${EVENTS[@]}"

section "Volatile findings past/nearing review_by (check dates!)" "research/ re-verify routine"
jq -s '[.[] | select(.event_type=="research.finding" and .payload.volatility=="volatile")
  | {topic: .payload.topic, review_by: .payload.review_by, claim: (.payload.claim[:100])}]' "${EVENTS[@]}"

section "Open research questions by topic" "future specs (owners)"
jq -s '[.[] | select(.event_type=="research.open_question")]
  | group_by(.payload.topic) | map({topic: .[0].payload.topic, n: length})' "${EVENTS[@]}"

section "Learning events (mistake ledger)" "SKILLS.md mastery evidence"
jq -s '[.[] | select(.event_type | startswith("learning."))] | length
  | if . == 0 then "none yet — starts with 002 construction" else . end' "${EVENTS[@]}"

#!/usr/bin/env bash
# goldeneye PostToolUse hook (Write|Edit): telemetry for spec-doc writes + gate
# notification when a gated doc reaches "awaiting-review".
# Notify-only v1: Telegram if TELEGRAM_BOT_TOKEN/TELEGRAM_CHAT_ID are set; approval
# happens in-session or by editing the Status line. Hooks must never fail the tool.
set -uo pipefail

INPUT="$(cat 2>/dev/null || true)"
command -v jq >/dev/null 2>&1 || exit 0
FILE="$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null)"
[ -n "$FILE" ] || exit 0

ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
REL="${FILE#"$ROOT"/}"

case "$REL" in
  specs/_template/*) exit 0 ;;
  specs/*/intent.md|specs/*/requirements.md|specs/*/design.md|specs/*/tasks.md|specs/*/evidence.md) ;;
  *) exit 0 ;;
esac

SPEC_ID="$(printf '%s' "$REL" | cut -d/ -f2)"
DOC="$(basename "$REL")"

# Telemetry: record the doc write (payload = file + doc, not the whole content).
printf '{"file":"%s","doc":"%s"}' "$REL" "$DOC" \
  | GOLDENEYE_SPEC_ID="$SPEC_ID" bash "$ROOT/.claude/hooks/telemetry.sh" spec.doc_written

# Gate notification: only gated docs, only when awaiting review.
case "$DOC" in
  requirements.md|design.md|tasks.md) ;;
  *) exit 0 ;;
esac
grep -qE '^\*\*Status:\*\*[[:space:]]*awaiting-review' "$FILE" 2>/dev/null || exit 0

MSG="goldeneye ✋ ${SPEC_ID}/${DOC} is awaiting your review"
if [ -n "${TELEGRAM_BOT_TOKEN:-}" ] && [ -n "${TELEGRAM_CHAT_ID:-}" ]; then
  curl -sf --max-time 10 "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendMessage" \
    -d chat_id="${TELEGRAM_CHAT_ID}" -d text="$MSG" >/dev/null 2>&1 \
    && CHANNEL="telegram" || CHANNEL="telegram-failed"
else
  CHANNEL="none-configured"
fi
printf '{"file":"%s","channel":"%s"}' "$REL" "$CHANNEL" \
  | GOLDENEYE_SPEC_ID="$SPEC_ID" bash "$ROOT/.claude/hooks/telemetry.sh" gate.notified

exit 0

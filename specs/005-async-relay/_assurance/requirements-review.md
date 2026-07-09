# Assurance audit — 005 async-relay / requirements.md

**Auditor:** spec-auditor (fresh context) · **Date:** 2026-07-09
**Doc audited:** `specs/005-async-relay/requirements.md` (Status: awaiting-review)
**Audited against:** `specs/005-async-relay/intent.md` only, plus the contracts it
invokes by name (`datalake/schema/envelope.v1.json`, glake's 003/004 requirements)
and CLAUDE.md pipeline rules.

---

## Summary

Strong draft. Readability standard met (plain-words opening, not-building,
learn-list, criteria below the fold, changelog collapsed). Traceability to intent
is clean — every criterion serves the distilled intent, no orphans, localhost-only
and no-AWS boundaries are explicit, and the learning goals (SKILLS 1d/2a, T1–T6)
are covered by A4/A6 (concurrency, shutdown) rather than left implicit. The 003/004
audit lessons visibly landed.

But three precision holes remain, and each is exactly the kind of thing the human
gate should not have to discover: what "valid" means is weaker than both the
envelope schema and the doc's own "strict gatekeeper" promise; the healthz
invariant is untestable as stated under concurrency; and A4's "no duplicated
lines" clause contradicts legitimately duplicate submissions. Plus a process gap:
no intent audit exists in `_assurance/`, yet the doc is `awaiting-review`.

---

## MAJOR

### M1 — "Valid envelope" is under-defined, and A8 makes the definition impossible as written
A1 defines valid as "all `REQUIRED_KEYS` present, top-level". Three problems:

1. **Weaker than the schema.** `envelope.v1.json` also constrains types:
   `schema_version` is `const: 1` (integer), `payload` is an object, `ts` is an
   RFC3339 date-time string. Presence-only validation admits
   `{"schema_version": 99, "payload": "oops", ...}` into the lake. The doc sells
   relay as a **strict gatekeeper** — either tighten, or explicitly declare
   type/const checks out of scope for v0 (one sentence in "What we're not
   building yet"). This is a decision the human should see, not infer.
2. **Tension with A8.** A8 says validation "comes from the learner's `glake` lib
   … not reimplemented". But glake's validation (003 R1: keys present, explicitly
   *not* checking `ts`) is strictly weaker than what A2 demands (reject bad-ts).
   Rejecting bad-ts therefore requires either relay-side logic (contradicting
   "not reimplemented" read literally) or glake exposing its ts→day rule
   publicly (a new demand on 004's API, unstated). Fix: rephrase A8 as "relay
   reuses glake's *classification* (valid / missing-key / bad-ts) via its public
   API and maps any non-valid class to 400" — and note this presumes glake's day
   extraction is public.
3. **The ts→day rule has no single source of truth here.** "a `ts` that yields no
   valid `YYYY-MM-DD` day" — yields *how*? Prefix slice? Full RFC3339 parse?
   `"2026-07-09"` (a date, not a date-time, schema-invalid) has a perfectly valid
   10-char day prefix. Cite glake's extraction rule by reference so relay and
   glake cannot disagree — A5's cross-check property depends on them agreeing.

### M2 — A3's counter invariant is untestable/false as stated under concurrency
"live counters `{received, accepted, rejected}` that add up" — always? With three
independent atomics, a `/healthz` read racing a handler can observe `received`
incremented before `accepted` is — the invariant transiently fails, so the
requirement as written is false for the obvious implementation and the [E] test
would only pass by probing at quiet moments. This is not pedantry: it's the T4
("shared state done right") lesson in miniature, and the doc should decide it.
Fix, one of:
- weaken: "at quiescence (no in-flight requests), received = accepted + rejected" — keep [E];
- strengthen: "every `/healthz` response satisfies received = accepted + rejected,
  including under concurrent load" — retag **[P]**, and accept that this forces a
  consistent-snapshot design (single atomic pair / counters owned by one task).

### M3 — A4's "no duplicated lines" conflicts with duplicate submissions; duplicate `event_id` policy is silent
The envelope schema declares `event_id` a "unique id", but nothing in the doc says
what relay does when two POSTs carry the same event (or same `event_id`, different
body). Presumably: accept both, dedupe is not relay's job — fine for a lab tool,
but say it, because as written A4 *breaks* on that case: if the generator (or a
retrying client) submits the same valid event twice, both correctly get 202 and
disk correctly holds two identical lines — which the current wording ("no …
duplicated lines"; "parses back as exactly one of the accepted events") calls a
failure. Fix A4's invariant to multiset equality: "the multiset of lines on disk
equals the multiset of bodies that received 202", and add one line stating the
duplicate-`event_id` policy (accepted; uniqueness is the emitter's contract).

---

## MODERATE

### M4 — Default lake is the *live* lake, which the shell hooks also append to — the single-writer guarantee is in-process only
A7's default is `datalake/raw-local`, and `.claude/hooks/telemetry.sh` appends via
`>>` to the *same file* relay writes (`dt=<day>/events.jsonl` — confirmed in the
hook source). The doc's whole pitch is "two writers on one file tear lines", yet
the default configuration recreates cross-*process* dual writers that the channel
pattern cannot protect against. Fix: (a) require A4/A5 property runs on a fresh
temp lake (intent already assumes this — make the requirement say it), and (b) one
honest sentence acknowledging the live-lake coexistence limit (POSIX `O_APPEND`
small-write atomicity is what you're actually leaning on there) — it's a good
teaching beat, and silently shipping the collision would be a spec hole.

### M5 — A2's "naming the first problem" is untestable as written
"First" by what order — `REQUIRED_KEYS` declaration order? Byte order in the body?
Two correct implementations disagree. Fix: "naming *a* problem" or fix the
precedence (not-JSON → missing key in `REQUIRED_KEYS` order → bad ts).

### M6 — A6 is compound (the pre-applied "single-clause EARS" lesson, missed once)
Five assertions in one line: stop accepting + in-flight finish + drain + flush +
exit 0 + all-202s-on-disk. Split: **A6a** (SIGINT → stops accepting, in-flight
finish, exits 0) and **A6b** (post-exit, every 202'd event is on disk — which is
really the shutdown clause of A4 and could simply cite it). Also a gap: what does
a request arriving *during* drain receive — connection refused? 503? Any answer is
fine; no answer means the A4 property's shutdown phase can't decide whether a
late-fired request's response counts.

### M7 — Process: no intent audit on file, yet the doc is `awaiting-review`
`specs/005-async-relay/_assurance/` does not exist; `intent-review.md` is absent
and the header's Assurance field is "—". CLAUDE.md: assurance before attention.
Run `intent-assurance` on intent.md, embed both verdicts in the combined-ack
notification. (I could not perform the mandated <80%-divergent-readings check
because there is no verdict to check.)

---

## MINOR

- **m1** — A ts with a *valid but weird* day (year 9999, 1970) silently creates an
  arbitrary `dt=` partition. Acceptable for a lab tool — one sentence saying so
  closes the hole.
- **m2** — No criterion exercises the intent's "hooks can POST": add a fixture [E]
  (one real hook-emitted envelope from `datalake/raw-local` round-trips to 202) —
  cheap, and it pins the compatibility claim.
- **m3** — Request body size limit unspecified (axum defaults to ~2 MB). Fine to
  inherit the default; say "framework default" so it's a decision, not an accident.
- **m4** — No line uses EARS keyword shapes (WHEN/IF-THEN/ubiquitous SHALL); house
  style matches approved 003/004, all lines are tagged and (post-fixes) testable,
  so noted for the record only.

## Tag & coverage check

- Tags present on all 10 lines. A4/A5 correctly [P]; A8–A10 correctly [O].
  A2 could arguably be [P] (generate invalid bodies), but A4's "rejected events
  appear nowhere" already covers the property side — [E] is defensible.
- Intent coverage: distilled intent ✔ (local axum, POST /events, dt= partitions,
  concurrent, validated, glake reuse); learning goal 1d/2a ✔ via T1–T6 + A4/A6;
  exclusions (AWS, TLS/auth, batching, queues) ✔ mirror intent's rejected
  readings. No orphan requirements.

---

**VERDICT: 68% — Well-shaped and fully traceable to intent, but the validity
definition (vs. schema and A8), the healthz consistency model, and the
duplicate-line clause of A4 must be pinned down before a human can approve
without inheriting three silent design decisions.**

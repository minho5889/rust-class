# Requirements Audit — 007 lake-to-s3

**Auditor:** spec-auditor (fresh context, adversarial)
**Date:** 2026-07-10
**Doc:** `specs/007-lake-to-s3/requirements.md` (rev: initial fast-path draft, awaiting-review)
**Audited against:** `specs/007-lake-to-s3/intent.md` only, plus the cross-references the
doc itself invokes (006 H-lines, 003 glake semantics, `datalake/README.md`, CLAUDE.md
conventions). Intent-review verdict was **93%** (≥ 80%), so no divergent-reading
open-questions obligation applies — checked and confirmed.

---

## MAJOR findings

### M1 — S11/S12 "agree with glake stats" repeats the exact naive-"matches" bug 003 had to fix in rev 4

Two independent reasons the stated identity is false as written:

1. **Scope mismatch (the 003 R10 lesson, unlearned).** `glake stats` totals the
   *whole* lake including `traces/` (003 R3a/R10: `221 = 157 + 64` — glake = process
   events + trace lines). The standing queries read `dt=*/events.jsonl` only. The
   local variants "must agree with glake stats totals" (S11) can only hold with a
   reconciliation identity, not equality — 003 R10 was amended from "matches" to the
   identity for precisely this reason.
2. **S12's identity breaks the moment ingest works.** S12 requires, in one line:
   "ingest deployed, one curl → one object in `raw/dt=…/`" *and* "DuckDB-over-S3 …
   agrees with `glake stats`." Ingested events exist **only in S3** (they never touch
   the local lake), so after the curl, S3 holds ≥1 more event than local `glake stats`
   can see. Equality is off by exactly the ingested count, by design.

Also latent in "agree": glake skips blank lines (003 R5) and does not count malformed
lines as events, while `read_json_auto` errors on malformed JSON unless
`ignore_errors=true`. The real lake is probably clean, but the criterion should
define the identity so it stays testable when it isn't.

Relatedly, the plain-words worked example bakes in the glob
`s3://goldeneye-lake/raw/dt=*/events.jsonl` — which **silently excludes ingest's
`evt-<event_id>.json` objects** under the same prefix (two writers, two filename
shapes). The scan pack must union both layouts or every query undercounts.

**Fix:** rewrite S11/S12's agreement clauses as explicit reconciliation identities
(e.g., S3-scan process-event count = local `dt=*` count + ingested-object count;
traces in/out stated; blank/malformed handling stated), and make S11 require the
queries to cover *both* key shapes under `raw/dt=*/` (or state ingest objects are a
separate query). Fix the worked-example glob to match.

### M2 — the `traces/` subtree is undefined under sync, and the real backlog contains it

The real lake has `datalake/raw-local/traces/dt=2026-07-05/02-collections-lens-28630.jsonl`
today. The plain-words mechanism is "walks your local lake with your own glake
walker" — which walks `traces/` (003 R3a, amended rev 4 to say exactly that). But
S1's normative mapping covers only `dt=<day>/<file>` → `raw/dt=<day>/<file>`; a path
shaped `traces/dt=<day>/<file>` matches no rule, and S1's "nothing is ever written
outside `raw/`" forbids the `s3://goldeneye-lake/traces/` destination that
`datalake/README.md` names for Wave 2 (that promise is traces-as-*Parquet*, which
this spec correctly rejects for now — but then raw trace JSONL needs a stated home
or a stated exclusion). S12's "real backlog synced" hits this file on deploy day
with no defined behavior; the S3 conservation property's input domain (generated
lakes) silently depends on the answer.

**Fix:** one explicit clause: either (a) traces sync verbatim to
`raw/traces/dt=<day>/<file>` (JSONL now, Parquet is the later unit), or (b) sync
skips `traces/` and the dry-run plan says so — and in either case S3's generator
domain and the S11/M1 identity must match the choice.

### M3 — the 006 supersession story is missing: two specs assert incompatible facts about the same crate

007 evolves `crates/hello-lambda` in place (intent assumption). S6 preserves 006 H4
(door equivalence) explicitly — good — but is silent on the two 006 lines the
evolution *contradicts*:

- **006 H1:** accepted event → "exactly one compact JSON line on stdout." 007's
  intent says events land as S3 objects "**instead of** log lines." Is the stdout
  emission removed, or kept alongside the put? S6 says "exactly one `put`" but says
  nothing about stdout — untestable ambiguity, and H1 remains a live claim in a
  gated doc.
- **006 H7:** "**No `aws-sdk-*`**" dependency policy on this crate. 007 adds
  `aws-sdk-s3` to it. S8 measures the delta but nothing marks H7 as superseded.

Per the change protocol, the amendment to 006 (changelog entries marking H1's sink
and H7's dependency policy superseded-by-007) must be part of this unit's stated
work, or the repo carries two gated specs claiming different sinks and dependency
rules for one deployable.

**Fix:** add a criterion (or a clause in S6): state whether stdout emission survives
(recommend: removed; CloudWatch keeps the `tracing` logs, the lake entry lives in
S3), and an [O] line requiring 006 H1/H7 supersession changelog entries at 007
approval.

### M4 — S3/S4 interplay: conservation after a re-sync of a changed file is unpinned

S3 asserts conservation "after sync" of a generated lake. S4 allows mutating a file
and re-syncing. Whether conservation still holds after that second sync depends
entirely on an unstated fake-store semantic: **does `put` overwrite the key (one
object per key, last write wins — real S3), or append an object?** An appending fake
satisfies S4's "uploads exactly the changed objects" while making the store's line
multiset double-count the file's old content — S3 and S4 would then be co-satisfiable
only on fresh lakes, and the law suite proves less than it claims.

**Fix:** state the stronger combined law: *after any sequence of local edits and
sync runs, the fake store's per-key content equals the current local lake's content*
(this subsumes S3 and S4's second half), and require explicitly that the fake models
S3 key semantics: put replaces, one object per key.

---

## MINOR findings

### m1 — S3's multisets are asymmetrically worded
"the multiset of **event lines** across the fake store's objects equals the multiset
of **local lines**" — "event lines" on one side, "local lines" on the other. If sync
copies files verbatim (single PUT), blank/whitespace lines exist on both sides and
the property should say "lines" (byte-verbatim) on both; if it filters, say what.
Pick one term, both sides — the [P] generator can't be written from this sentence.

### m2 — compound criteria (house precedent: 003's audit forced R3→R3a/b splits)
- **S1** carries three normative claims: sync mapping, ingest key, never-outside-`raw/`.
  Split S1a/S1b/S1c. Note "nothing is **ever** written outside `raw/`" is a
  universal invariant wearing an [E] tag — it's [P]-shaped (assert over the fake's
  ledger for all generated inputs, all commands) or should be folded into S3's law.
- **S5** is two distinct failure scenarios (unreadable path; mid-run put failure) →
  S5a/S5b.
- **S6** is three clauses (accepted→one put; rejected→zero puts; door unchanged/H4).
  Tolerable as a family, but S6a/S6b/S6c would match house style.
- **S12** packs six deploy-day checks into one [O]; 006 split its deploy day into
  H9/H10/H11. Splitting makes partial deploy-day evidence recordable.

### m3 — `goldeneye-discovery` is created and then never used by any requirement
It traces to intent (both allowed hardcoded names, CLAUDE.md-compliant), but no S
line reads, writes, or verifies it beyond existence, and its purpose (Wave-3 insight
cards per `datalake/README.md`) is nowhere stated. An unused, termination-protected
resource with no stated purpose is a reviewer trap. **Fix:** one sentence in S10
naming its Wave-3 purpose and the create-now-use-later rationale (create both
hardcoded buckets once), and confirm S10's security posture (SSE-S3, public-access
block, RETAIN) applies to *both* buckets.

### m4 — T5 "content-derived keys" contradicts S1's path-derived keys
S1's keys are path-derived (`raw/dt=<day>/<file>`); idempotence comes from
etag/size *comparison*. T5's "content-derived keys" would send the design toward a
different (content-addressed) layout that S1 forbids. Reword: "content-derived
**change detection** (etag/size) over path-derived keys."

### m5 — T4 (streaming + buffer reuse) and the plan's "batching" have no acceptance criterion
Every other T maps to an S line. Buffer reuse is verifiable in this repo of all
places (code review [O], or a memlens trace showing no per-line allocation). Add an
[O] or drop the promise; note intent's plan line says "batching, buffer reuse,
streaming serde" and only S9's concurrency bound gestures at batching.

### m6 — the retained-bucket cost decision isn't surfaced in plain words
Intent promises the ongoing cents-level cost is "flagged to the learner at the gate
(AI proposes, human disposes)." S10/S12 encode RETAIN, but the plain-words section
never says "these two buckets stay after teardown and cost ~cents/month — your
call." One sentence restores the promise at the layer the human actually reads.

### m7 — `<day>` domain undefined; `dt=bad-ts` is a legal local partition
005's writer quarantines to `dt=bad-ts`; nothing stops such a dir existing locally.
S3 accepts any key, but S1 doesn't say whether sync mirrors partition names
literally (recommended: sync never judges, it mirrors — bad-ts included) or
validates them. The [P] generator domain needs this answer; one clause fixes it.

### m8 — concurrent sync runs unaddressed
Same-content last-write-wins makes two simultaneous runs harmless in practice, but
the doc should say "single-invocation tool; concurrent runs out of scope" so the
idempotence law's quantification ("an immediate second sync") is clearly sequential.

### m9 — Graviton crypto checklist not cited for the etag compare
If change detection computes MD5 locally (the etag comparand) via a hash crate, the
constitution's ARM64-hardware-backend checklist applies to these ARM64-built
binaries (the sha2 4–5× lesson). Add a clause to S7 or note it for design.

---

## What holds up

- **Traceability/coverage vs intent:** all four intent elements (stateful stack,
  lake-sync, ingest evolution, scan pack), the learning goal (2e/2a/1c), and the
  headline SDK-size measurement (S8) each land on criteria; no orphan requirements
  beyond m3's unused bucket. Rejected interpretations are respected (no Parquet, no
  two-way sync, no deploy-from-session).
- **Scope tags** are honest and consistent with the no-credentials reality; the
  honesty box matches 006's approved pattern.
- **Tag discipline** is mostly sound ([P] on the two laws, [O] on build/infra
  facts); the declarative single-fact style matches approved house EARS usage.
- **Constitution compliance:** hardcoded names limited to the two allowed; S11
  carries the queries-name-their-doc telemetry rule; S10's IAM scoping
  (`s3:PutObject` on `goldeneye-lake/raw/*` only) is exactly right.
- **Readability:** console-first plain words, honesty box, not-building list — the
  house standard, well executed.

## Scoring rationale

Structure, traceability, and readability are gate-ready, but four MAJORs mean a
human reviewing today would be approving criteria that are false-as-written
(M1's identities), undefined on real data the spec itself will touch (M2), or
contradictory across gated docs (M3) — and one [P] whose law suite is unsound
without an unstated fake semantic (M4). All four have small, local fixes.

VERDICT: 68% — Traceable, honest, and readable, but not yet approvable: S11/S12's "agree with glake stats" is false as written (the 003 rev-4 lesson plus ingest's S3-only events), sync's behavior on the real `traces/` subtree is undefined, the 006 H1/H7 supersession is unstated, and S3/S4's conservation law needs the fake's put-overwrites-key semantic pinned.

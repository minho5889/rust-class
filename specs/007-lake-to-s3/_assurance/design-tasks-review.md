# Design + Tasks Audit — 007 lake-to-s3 (combined)

**Auditor:** spec-auditor (fresh context, adversarial)
**Date:** 2026-07-10
**Docs:** `specs/007-lake-to-s3/design.md` (vs `requirements.md`) and
`specs/007-lake-to-s3/tasks.md` (vs `design.md`), both rev "initial fast-path
draft", both `awaiting-review`.
**Also consulted:** 007 intent + `_assurance/requirements-review.md` (68%, 4 MAJORs,
un-revised), 006 design + `_reference/hello-lambda/`, 005 design,
004 `_reference/glake/src/walk.rs`, CLAUDE.md conventions, repo state
(`crates/`, `datalake/queries/`, absence of any 007 `_reference/`/`sittings/`).

---

## MAJOR findings

### M1 — the core teaching artifact will not compile as designed: bare `async fn in trait` + generic core + `JoinSet::spawn` has no `Send` story

`run.rs` is specified as `JoinSet` + `Semaphore(4)` executing puts against generic
`S: ObjectStore`, where the trait is bare `async fn put(...)`. `JoinSet::spawn`
requires the spawned future to be `Send + 'static`. On stable Rust there is **no
way to bound the `Send`-ness of a bare AFIT method's returned future** from a
`S: ObjectStore` generic (return-type notation is unstable). The compiler will
reject the spawn with "future returned by `put` cannot be sent between threads" —
for *every* `S`, including the fake. The same question chases the evolved handler:
S6's fake-store tests only work if the handler is generic over `ObjectStore`, and
`lambda_http`'s service plumbing raises the identical bound issue.

The design's honest-note ("generic-only, no `dyn`, exactly what sync needs") is
right about `dyn` and silent about `Send` — the half that actually bites.

**Fix (pick one, and write it into the store.rs row + a decision-row amendment):**
(a) desugar to stable RPITIT with the bound spelled:
`fn put(&self, ...) -> impl Future<Output = Result<(), StoreError>> + Send;`
(this is *the* AFIT teaching moment — take it); or (b) `trait_variant::make(Send)`;
or (c) drop `JoinSet` for `StreamExt::buffer_unordered(4)` driven on one task (no
`Send` needed — but then the Semaphore/gauge story changes and S9's design shifts).
Also pin the `'static`/ownership shape the spawn forces: `Arc<S>` (or `S: Clone`),
owned `String` keys, and `Arc<Semaphore>::acquire_owned` moved into the task.

### M2 — etag realism gap: the real S3 ETag is *quoted*; the fake will never catch it, and deploy-day idempotence (`uploaded 0`) silently fails

`plan.rs` compares local md5 hex against remote etag. S3's ListObjectsV2 returns
`ETag` **wrapped in double quotes** (`"\"d41d8...\""` — the SDK's `e_tag()` includes
them). The fake's author will naturally return bare hex; the properties run only
against the fake (that's the point), so nothing in-session can catch a missing
quote-strip in `S3Store::list` — and S12's headline demo, re-run shows `uploaded 0`,
fails on the learner's account with every file re-uploading forever. Three
sub-gaps, one cluster:

1. **Quote normalization is nowhere in the design.** Pin it: `S3Store::list` strips
   quotes (or `plan` normalizes both sides), with a comment naming why.
2. **The fake's etag behavior is unstated.** `fake.rs` is specced as
   `Mutex<BTreeMap<String, Vec<u8>>>` + counters — no etag. For `plan` to work at
   all, the fake's `list` must compute md5-of-stored-bytes in the *same normalized
   format* `plan` expects. Say so in the fake.rs row.
3. **The S4 mutation arms never exercise the etag branch.** Both arms
   ({append a line, touch new file}) change *size*, and size is checked first —
   the md5 path is dead code under the property as generated. Add a same-size arm
   (flip one byte / replace a line with an equal-length line) so the etag compare
   is actually load-bearing in the law.

Also tie the etag=md5 premise to encryption explicitly: it holds for single-part
PUT under **SSE-S3** (which S10 mandates) and breaks under SSE-KMS — one sentence
next to the multipart caveat keeps the idempotence design honest.

### M3 — the cross-crate wiring is undrawn: how does `hello-lambda` reach `ObjectStore`/`S3Store`/`FakeStore`?

The architecture diagram shows the ingest handler using the same trait and both
impls, and fake.rs is "`pub` in lib for the ingest crate's tests too" — but no
dependency edge is stated anywhere. The only reading is `hello-lambda` taking a
path dependency on the `lake-sync` **package**, which:

- drags `clap`, `md5`, glake-walk, and the CLI's whole dep set into the Lambda's
  build graph (cargo doesn't scope package deps to the bin target) — **confounding
  S8's headline measurement**, which is supposed to isolate what `aws-sdk-s3` costs
  the 006 binary;
- inverts layering (a deployable depending on a CLI tool) when the constitution's
  workspace explicitly reserves `crates/shared/` for exactly this;
- ships a test fake in the production lib surface with no `test-support`
  feature-gate story.

**Fix:** extract `store.rs` + `fake.rs` + `s3.rs` into a small crate (e.g.
`crates/lake-store` or `crates/shared`), depended on by both `lake-sync` and
`hello-lambda`; or, if the single-crate shape is kept deliberately, draw the edge
in the shape table, gate the fake behind a feature, and state the S8 confound and
how the measurement subtracts it. Either way this is a design decision missing its
decision row.

### M4 — design+tasks were built on requirements that failed their own audit (68%), and the four upstream MAJORs arrive here unresolved

`_assurance/requirements-review.md` flagged four MAJORs; requirements.md shows no
revision. The design inherits, not fixes, three of them and half-fixes the fourth:

- **traces/ (req-M2):** `plan.rs` walks with glake's `jsonl_files`, which recurses
  into the real `datalake/raw-local/traces/` subtree — and the design's key mapping
  still only defines `dt=<day>/<file>`. Cheapest fix is to define the key as
  `raw/` + path-relative-to-root (which handles `traces/` verbatim) — the design
  never says how keys are derived from paths at all.
- **scan-pack identity + glob (req-M1):** the ingest-key decision row claims
  "DuckDB globs both shapes with one pattern" but the pattern is never given
  (requirements' worked example still shows `dt=*/events.jsonl`, which excludes
  `evt-*.json`). Pin it — e.g. `raw/dt=*/*.json*` with `format='newline_delimited'`
  — and state the invariant that makes it valid: ingest bodies are exactly one
  compact JSON line. And the S12 "agrees with glake stats" identity (broken by
  S3-only ingested events and by traces) is still nowhere reconciled.
- **006 supersession (req-M3):** the sink swap contradicts 006 H1 (stdout line) and
  H7 (no `aws-sdk-*`). Neither design (does stdout emission survive?) nor tasks
  (no 006-amendment work item in 1.6 or 2.1) carries the change-protocol work.
- **fake put-replaces semantics (req-M4):** implicitly resolved (`BTreeMap` insert
  replaces) but the law is never stated, and conservation-after-mutate+re-sync is
  still unasserted — one extra `prop_assert!` in S4's mutation arm (re-check the S3
  multiset equality after the second sync) closes it nearly for free.

Per the pipeline's change protocol, revise requirements first (or in the same
combined rev) — approving this design would freeze answers to questions the
requirements doc still asks wrong.

### M5 — both docs assert validation that does not exist in the repo

design.md: "**the reference was validated with all of these**" (past tense);
tasks.md: materials "are authored and validated against the 007 reference
**before this gate is presented**." The spec directory contains five markdown
files and nothing else — no `_reference/`, no `sittings/`, no scan-pack SQL, no
`crates/lake-sync`. 006 at the same stage had its full `_reference/` crate
in-tree. With status already `awaiting-review`, these are unevidenced claims in
gated docs. **Fix:** land the reference + materials before presenting the gate, or
reword both passages to future-tense plan and add the validation as an explicit
pre-gate task.

---

## MODERATE findings

### D1 — S9's gauge test can pass vacuously (and the design doesn't prevent it)

If the fake's `put` returns without an await point, tasks complete near-instantly
and the high-water mark may never exceed 1 — the `≤ 4` assertion then passes even
if the Semaphore is deleted. The fake must **hold the gauge across an await**
(`yield_now`/short sleep between increment and decrement), the permit must be held
across the whole put, and the test should assert `2 ≤ hwm ≤ 4` under the many-file
lake so both "concurrency is real" and "the bound holds" are proven. None of this
is in the fake.rs row or task 1.4.

### D2 — S5's mid-run put-failure clause has no mechanism: the fake can't fail

`fake.rs` (BTreeMap + counters) has no failure injection, but S5 requires testing
"a put that fails mid-run → object named on stderr, non-zero exit, partial
progress reported." Add a fallible arm to the fake (fail-on-key or fail-after-N)
to its design row, and note that the partial-progress logic lives in `run.rs` —
whose row cites S9/T4 but not S5 (traceability gap: no element owns S5's
mid-run-failure semantics; `plan.rs` is pure and `main.rs` only formats).

### D3 — `grantPut` grants more than S10's "s3:PutObject … and nothing wider"

CDK's `Bucket.grantPut` emits `s3:PutObject` **plus** the `PutObject*` variants
(tagging/retention/legal-hold) **and `s3:Abort*`**. The claimed template assert
("the scoped `raw/*` grant") will either fail a strict check or quietly loosen the
requirement. If "nothing wider" is literal, hand-roll one `PolicyStatement`
(`s3:PutObject` on `arn:aws:s3:::goldeneye-lake/raw/*`) via `addToRolePolicy`;
otherwise amend S10's wording. (Cross-stack direction itself is sound: stateless
imports the bucket from stateful, grants land on the function role, no cycle; the
CFN-export lock only binds while stateless exists, which same-day teardown
releases — worth one comment in the stack file.)

### D4 — one-way-sync domain assumption is load-bearing for the conservation law and unstated

Sync never deletes remote objects (intent: one-way). If a local file shrinks or
disappears, the store keeps orphaned content and S3's "store lines = local lines"
is simply false. The properties survive only because the generator never deletes —
that's fine, but say it: *the local lake is append-only by construction (hooks
append); delete/shrink is outside the conservation law's domain, and orphaned
remote objects are accepted semantics.* One sentence in the Properties section
makes the law's quantifier honest.

## MINOR findings

- **m1 — OnceLock vs the unwrap ban:** `OnceLock` is sync, the SDK client build is
  async — so it's set-in-main / `get()`-in-handler, and `get()` returns `Option`
  in crates where `unwrap_used`/`expect_used` are denied. Name the intended
  pattern (closure-capture of a `Clone` store into `service_fn` — SDK clients are
  Arc-backed — or `tokio::sync::OnceCell`); 006's `OnceLock` precedent was
  sync-init and doesn't transfer cleanly.
- **m2 — who-writes drift:** requirements T1 says the learner "implement[s] it
  twice: FakeStore … S3Store"; task 1.5 makes S3Store "read together line by
  line." Defensible under co-write-the-hard-bits, but reconcile the promise.
- **m3 — ops checklist omits S7:** the API-rubric review (1.1 "groundwork") never
  reappears; add "S7 rubric pass recorded" to the checklist so the [O] is closable.
- **m4 — hello-lambda row doesn't cite S1** though the ingest key is S1-normative
  (currently reached only via S6's "its S1 key").
- **m5 — lake-sync's aarch64 build has no stated purpose** — it runs on the dev
  machine, not Graviton. Fine as a cross-build proof for the aws-lc-rs decision
  row, but say that, or S8's artifact list looks like cargo-culted ARM64.

## What holds up

- The seam-and-fake architecture is the right teaching shape, and the decision
  table is genuinely argued (localstack vs mocks vs own-trait; list+compare vs
  manifest; per-file vs per-event keys; aws-lc-rs with a written fallback — all
  real alternatives with real reasons).
- Requirement→element traceability is otherwise tight: S1–S12 all land on shape
  rows or verify bullets, no scope-creep elements (every row cites REQs).
- Tasks: complete S-coverage across sittings R–U, properties genuinely red-first
  (1.2 before 1.3), one-concept-per-sitting pacing, deviation honestly declared,
  close-out includes property-auditor and the datalake README flip.
- "Rust you learn" column maps cleanly to T1–T6 and SKILLS 1c/2a/2e; the F13
  generic-vs-dyn contrast and the 005 backpressure echo are real curriculum
  threading.
- Generator realism in S3 (blanks/malformed included, "sync moves bytes, not
  judgments") quietly fixes requirements-m1.

## Scoring rationale

The pedagogy and structure are gate-ready; the engineering is not. M1 is a
guaranteed compile failure at the exact artifact the unit exists to teach; M2 is a
silent deploy-day failure of the headline demo that the chosen test strategy
structurally cannot catch; M3 leaves the crate graph — and the S8 measurement's
validity — undefined; M4 means approving this design would freeze wrong answers
from an un-revised 68% requirements doc; M5 is an honesty defect in gated text.
All fixes are local and none threaten the architecture.

VERDICT: 58% — Right architecture and well-shaped tasks, but not approvable yet: the bare-AFIT generic core can't satisfy JoinSet's Send bound as written, the quoted-ETag mismatch defeats deploy-day idempotence invisibly to the fake-only test strategy, the hello-lambda↔lake-sync crate wiring (and its S8 confound) is undrawn, four upstream requirements MAJORs arrive unresolved, and both docs claim a validated reference that isn't in the repo.

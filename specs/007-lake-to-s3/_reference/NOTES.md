# 007 lake-to-s3 — reference implementation notes

Validation record for the three reference crates (`lake-store/`,
`lake-sync/`, `hello-lambda/`), the `infra/` stateful+stateless changes,
and the DuckDB scan pack (`datalake/queries/duckdb/`), authored
2026-07-10. Everything below ran in this session; exit codes were captured
directly (no pipe-masking).

Toolchain: rustc/cargo 1.94.1 · cargo-lambda 1.9.1 (zig via `ziglang`) ·
aws-sdk-s3 **1.138.0** / aws-config **1.9.0** · md5 0.8.1 · proptest 1 ·
duckdb 1.5.4 (python module) · aws-cdk-lib 2.261.0 + cdk-nag 3.0.1 ·
node 22.22.2 · aarch64-linux-gnu-gcc (Debian cross toolchain).

---

## 1. The S8 bill (the headline measurement)

Both artifacts built for real ARM64 under the replicated
`[profile.release]` (lto=thin, codegen-units=1, panic=abort,
strip=symbols — same flags as the workspace root, or the number lies).

| Artifact | Bytes | MiB | How built |
|---|---|---|---|
| 006 baseline `bootstrap` (no SDK) | **1,843,256** | 1.76 | `cargo lambda build --release --arm64` (006 session; re-verified on disk) |
| **007 evolved `bootstrap`** | **11,431,848** | **10.90** | same command, this session, exit 0 |
| **Δ — what aws-sdk-s3 + aws-config cost** | **+9,588,592** | **+9.14** | ×6.2 the baseline |
| lake-sync (ARM64, for scale) | 11,435,352 | 10.90 | `cargo build --release --target aarch64-unknown-linux-gnu`, exit 0 |

- The research expectation was "~10 MB+" — measured **+9.14 MiB**: no wild
  departure; the constitution's number survives contact with reality.
- Evolved sections (`aarch64-linux-gnu-size`): text 10,848,345 ·
  data 579,584 · bss 17,144 (006: text 1,767,224 · data 72,984 · bss 5,208).
- `file`: ELF 64-bit LSB pie, ARM aarch64, dynamically linked, stripped —
  same shape as 006; cargo-lambda's zig link, no fallback needed.
- The cdk.out bundled asset is byte-identical (11,431,848) — cargo-lambda-cdk
  ran the same local build during synth.
- lake-sync cross-build note: **aws-lc-sys compiled clean for aarch64 with
  nothing but the `.cargo/config.toml` linker line**
  (`linker = "aarch64-linux-gnu-gcc"`); the design's documented
  rustls/ring fallback and CC/AR overrides were NOT needed in this
  environment.

### Where the 9 MiB lives (`cargo bloat --release --crates`, host x86-64 — bloat cannot cross-compile; approximation)

Evolved hello-lambda, top crates (host .text 10.1 MiB):

| Crate | .text share | Size | Reading |
|---|---|---|---|
| std | 18.7% | 1.9 MiB | (006: 519.9 KiB — more of std is instantiated now) |
| **aws_lc_sys** | 12.5% | 1.3 MiB | AWS-LC crypto, C, pregenerated bindings |
| h2 | 9.4% | 970.3 KiB | HTTP/2 — S3 talks h2; 006's runtime-API client was h1 |
| aws_smithy_runtime | 5.6% | 578.8 KiB | the SDK's client machinery |
| rustls | 5.1% | 529.2 KiB | TLS state machine (aws-lc-rs as provider) |
| hyper | 4.4% | 452.4 KiB | grown from 63.9 KiB in 006 (h2 features) |
| aws_config | 3.2% | 335.3 KiB | credential/region chain |
| aws_sdk_s3 | 3.2% | 327.5 KiB | the actual S3 API surface |
| ring | 2.3% | 236.3 KiB | second crypto lib, pulled via rustls's default feature set |
| aws_smithy_types | 2.2% | 226.2 KiB | |

Reading for the learner: the delta is **dominated by the transport stack —
TLS/crypto (aws_lc_sys + rustls + ring ≈ 2.1 MiB) and HTTP/2 (h2 + hyper
growth ≈ 1.4 MiB) plus the smithy runtime — not by the S3 API itself**
(aws_sdk_s3 is only ~330 KiB). You pay for a production-grade encrypted
HTTP/2 client; the bucket calls are almost free on top. Two crypto
libraries riding together (aws-lc-rs AND ring) is a known SDK feature-set
artifact — a future size hunt could unify them.

lake-sync's table is the same story plus clap_builder (203.1 KiB) — and it
never rides in the Lambda, which is exactly why the three-crate layout
exists (S8 de-confounding).

Graviton checklist (S7): the only hash/crypto **our code** performs is md5
via the pure-Rust `md5` crate — compatibility checksum, not security, no
hardware backend to verify: **n/a by construction, recorded**. (The SDK's
TLS uses aws-lc's own aarch64 paths — its business, not our checksum's.)

## 2. Crate validation (final pass, exit codes captured)

| Check | lake-store | lake-sync | hello-lambda |
|---|---|---|---|
| `cargo fmt --check` | 0 | 0 | 0 |
| `clippy --all-targets -- -D warnings` (default features) | 0 | 0 | 0 |
| `clippy` `--features fake` / `--all-features` | 0 / 0 | — | — |
| `cargo test` | **3** (default) + **6** (`--features fake`) | **15** | **13** |

Test census:

- **lake-store** — 3 StoreError C-GOOD-ERR tests (Send+Sync+'static,
  display style, source chain); with `fake`: + put-replaces-key,
  bare-hex-md5 etags (pinned to the d41d8… empty-input constant),
  fail_on injection. No doc-tests (examples are `text` blocks).
- **lake-sync** — 5 unit (S1 key table incl. `traces/` + `dt=bad-ts`
  verbatim; single-file root + prefix normalization; size-first/
  etag-second table; missing-root error; byte-exact read_body) ·
  8 examples in tests/cli.rs (usage exit 2 ×2, unreadable path exit 2,
  S2 dry-run offline with scrubbed env ×2 runs, prefix normalization,
  S2 ledger-zero at lib level, **S5 fail_on mid-run** with named object +
  partial progress, **S9 gauge** high-water ∈ [2,4] over a 24-file lake) ·
  **2 properties × 512 cases** (S3 conservation, S4 idempotence with
  {append, same-size, new-file} mutation arms + put-log exact-changed-set
  + post-mutation conservation re-check). Suite runs in ~4 s.
- **hello-lambda** — 5 unit (door fixed-order table, multiline
  re-serialization, **S6 ingest_key** shape/sanitization/non-string-id;
  state clock + OnceLock, unchanged from 006) · 7 examples (S6 one-put-at-
  the-S1-key, S6 multiline→same key, S6 retry-overwrites, H2 rejects put
  nothing, S6 put-failure→500, H3 healthz deltas, H3 404s) ·
  **H4 property re-run, 512 cases** against the live relay router —
  green post-evolution: the door did not move. No counterexamples; no
  `proptest-regressions/` created anywhere.
- `cargo tree -e normal | grep -c aws-sdk` = **4** for the evolved crate
  (aws-sdk-s3 + aws-sdk-sso/ssooidc/sts — the latter three are
  aws-config's credential providers). 006's crate remains untouched at 0.

Red-first note: the S3/S4 properties were written against the design's
generator spec before plan/run existed in final form, and they CAUGHT a
real bug on their first green attempt's precursor — see §4, decision 14
(the empty-relative-path key bug the S1 unit test also pinned).

## 3. infra synth + cdk-nag transcript

```console
$ npx tsc --noEmit                                            # exit 0
$ npx cdk synth -c helloLambdaDir=../specs/007-lake-to-s3/_reference/hello-lambda
#                                                             # exit 0, BOTH stacks
$ cdk.out/validation-report.json → "pluginReports": []        # zero unacknowledged
```

Template assertions (read back from cdk.out):

- **goldeneye-stateful**: both buckets `goldeneye-lake` /
  `goldeneye-discovery`; SSE `AES256` (SSE-S3); PublicAccessBlock all
  four true; **no** VersioningConfiguration (off); DeletionPolicy AND
  UpdateReplacePolicy `Retain`; two enforceSSL bucket policies (deny on
  `aws:SecureTransport=false`); manifest `terminationProtection: true`. ✓
- **goldeneye-stateless**: `LAKE_BUCKET=goldeneye-lake` env on the
  function; `Architectures: ["arm64"]`; role's DefaultPolicy holds exactly
  one hand-written statement — Sid `GoldeneyeLakeRawPutOnly`,
  `s3:PutObject` on `arn:aws:s3:::goldeneye-lake/raw/*` (grantPut
  rejected; its bundled extras would falsify S10's claim). ✓

### cdk-nag: what actually fired (discovery run, acknowledgments disabled)

| Rule ID (as reported) | Level | Resource |
|---|---|---|
| `AwsSolutions-IAM5[Resource::arn:aws:s3:::goldeneye-lake/raw/*]` | ERROR | HelloLambda/ServiceRole/DefaultPolicy |
| `AwsSolutions-S1` | ERROR | goldeneye-stateful/LakeBucket |
| `AwsSolutions-S1` | ERROR | goldeneye-stateful/DiscoveryBucket |

All three acknowledged with written reasons (IAM5 in
`lib/stateless-stack.ts` — the wildcard IS the least privilege, per-event
keys make a finite resource list impossible, one action + one prefix;
S1 ×2 in `lib/stateful-stack.ts` — single-operator BPA'd SSL-enforced lab
lake; an access-log bucket would itself flag S1 and double the stateful
footprint). 006's three stateless findings remain acknowledged unchanged.
No S3-related finding from the Serverless pack; no SSL finding fired
(enforceSSL satisfies AwsSolutions-S10).

**006's rough edge, confirmed twice more:** the granular IAM5 id embeds
the ARN's `::`s, which `Validations.acknowledge()` rejects (qualifyId
reserves the delimiter) → recorded via the public
`Validations.ACKNOWLEDGED_RULES_METADATA_KEY` channel, exactly like 006's
IAM4. And the CLI's suggested acknowledgment strings are wrong in both
directions: it suggests `'AwsSolutions::AwsSolutions-S1'` (the
`Pack::RuleId` form cdk-nag 3.0.1's matcher does not match) while the
bare `AwsSolutions-S1` via `Validations.of(bucket).acknowledge()` works.

## 4. Semantic decisions beyond the spec text (these feed the gate)

1. **Trait desugared, impls sugared.** The TRAIT is spelled exactly as
   design.md pins it (RPITIT `+ Send`, `Send + Sync + 'static` bound).
   The two IMPLS use `async fn` — clippy's `manual_async_fn` (under the
   mandated `-D warnings`) forbids the desugared form where it says
   nothing new, and the compiler still CHECKS each impl's concrete future
   against the trait's written Send promise (refinement). Taught in a
   comment at both impls: desugar where the desugaring says something.
2. **The fake's ledger counts only successful puts**; `fail_on` stores
   nothing and logs nothing (like a PUT that died on the wire), and the
   in-flight gauge decrements on both paths. `put_log()` (ordered keys)
   exists so S4 can assert the *exact changed set*, not just a count.
3. **The fake's put dwells across two `yield_now`s** so "in flight" is
   observable: without an await inside put, puts start and finish within
   one poll and the S9 gauge would read 1 forever — a vacuous bound. The
   gauge test asserts high-water ≤ 4 (the law) AND ≥ 2 (the measurement
   is alive).
4. **`lake-store::s3` re-exports `aws_sdk_s3::Client` as `S3Client`**, so
   consumers build the client without naming the SDK in their manifests —
   S7's "only in the real impl module" extended to the dependency graph.
5. **Dry-run is offline by structure** (S2): the binary's `--dry-run`
   path constructs no client, loads no credentials, and plans against an
   EMPTY remote listing; `--bucket` is accepted and ignored. Consequence,
   stated in main.rs: a dry-run plan is the first-sync upper bound — it
   cannot know what a populated bucket would skip. The CLI test proves
   the no-AWS claim with a fully scrubbed environment (`env_clear`).
   The ledger half of S2 (zero puts with a live listing) is a lib-level
   test against the fake.
6. **Exit-code boundary** (S5): plan-phase problems (bad usage, missing/
   unreadable path, LIST failure, md5-read io) → one stderr line, exit 2;
   run-phase PUT failures are *data*, not errors → Outcome, partial
   progress named (every uploaded key printed when failures exist), exit 1.
   A mid-run local *read* failure follows the io discipline (exit 2) —
   in-flight uploads may be cancelled by the early return, which is safe
   precisely because sync is idempotent: the next run heals.
7. **Success output is quiet, failure output is loud**: clean runs print
   exactly `uploaded N, skipped M` (the requirements transcript's shape);
   only failing runs enumerate uploaded keys + `FAILED n`. Dry-run prints
   per-key `upload …`/`skip …` lines + `plan: upload N, skip M` — without
   the transcript's `(301 events)` decoration (counting events means
   parsing content; sync moves bytes — glake counts events). Friction §6.1.
8. **Permit acquired in the producer loop, moved into the task**
   (`acquire_owned`): bounds in-flight puts (S9) AND resident bodies
   (≤ 4+1) — acquiring inside the task would bound only the puts while
   buffering every body in spawned-but-waiting tasks. Documented in
   run.rs as the memory half S9's gauge can't see.
9. **hello-lambda put-failure → 500** `{"error":"lake unavailable"}`
   (relay's writer-unavailable analog): a 202 after a lost put would be a
   durability lie. Counters: such a request is neither accepted nor
   rejected, so `received = accepted + rejected` holds on failure-free
   runs only — pinned by a dedicated test and said in the healthz docs.
10. **The `Emit` trait is deleted, not deprecated** (S6b made structural):
    the sink is now async and RPITIT traits aren't dyn-friendly, so the
    handler core went generic (`handle_with<S: ObjectStore>`) — FakeStore
    in tests, S3Store in main, monomorphized (F13's second appearance).
    H4's harness now observes the put ledger instead of a Vec sink.
11. **`ingest_key` sanitizes the event_id** to `[A-Za-z0-9._-]` (else
    `_`): an id containing `/` would nest the object a level deeper and
    silently escape S11's pinned glob `raw/dt=*/*.json*`. Hook-minted ids
    are digits-and-dashes — in practice a no-op. Non-string (but present)
    event_ids get their compact JSON rendering — presence is what the
    door checks; the spelling decision is documented at the extraction.
12. **No cross-stack reference**: the stateless stack sets
    `LAKE_BUCKET=goldeneye-lake` and writes the ARN as a literal, rather
    than importing the bucket construct — the hardcoded name IS the
    cross-spec contract, and zero CloudFormation coupling means compute
    teardown can never tangle with lake state. `enforceSSL: true` was
    added beyond S10's letter (free, one bucket policy, satisfies
    AwsSolutions-S10 so no suppression needed).
13. **Scan pack pins its schema**: `read_json(columns={…})`, never
    `read_json_auto` — auto-inference SAMPLES, and `payload`'s inferred
    type flipped between JSON and MAP(VARCHAR,JSON) run-to-run as the
    lake grew. Also learned: DuckDB's `->>` binds looser than comparison
    in a WHERE conjunction — `(payload->>'k') = 'v'` needs the parens;
    and a SELECT-list alias whose expression contains a subquery can't be
    lateral-referenced (hence the CTE in 08). Local/s3 variants are
    generated from one source so they can only differ in paths+preamble.
14. **Bug caught by the tests before it shipped** (kept as a lesson):
    `path.strip_prefix(root)` on a single-file root *succeeds with an
    empty remainder*, so the key became a bare `raw/`. The S1 unit test
    caught it; the fix (fall back to the file name on empty relative
    path) is commented at the site. This is exactly the class of edge the
    walker's "a file is walked as itself" rule (003 R3b) creates.

## 5. Scan pack outputs + the reconciliation identity (S11)

All eight `local/` queries executed in-session against the REAL
`datalake/raw-local` via duckdb 1.5.4 (exit 0 each; full tables retained
in the session transcript). Summary at the measurement instant
**2026-07-10T23:24Z**:

- **01 inventory** (feeds: sanity): 350 events, 10 types — top:
  spec.doc_written 142, gate.notified 108, research.finding 33,
  session.start 28.
- **02 gate funnel** (MEMORY.md): notified/approved per spec —
  002: 8/3 · 003: 13/2 · 004: 24/0 · 005: 24/0 · 006: 18/0 · 007: 21/0.
  (The 0-approved rows are the pipeline's real backlog signal: nothing
  past 003 has recorded a gate.approved event yet.)
- **03 doc churn** (MEMORY.md): 25 doc files; top churn
  002/tasks.md 16 · 002/design.md 12 · 004/tasks.md 10.
- **04 volatility** (research/): annual 22 · stable 9 · volatile 2.
- **05 review queue** (research/): both volatile findings are
  `typescript-cdk-for-goldeneye`, review_by 2026-10-05 — not yet due.
- **06 open questions** (future specs): rust-best-practices 4 ·
  rust-on-aws-compute 4 · typescript-cdk 4 · rust-study-materials 1.
- **07 learning ledger** (SKILLS.md): 0 — starts when coached sittings log.
- **08 reconciliation** (this spec's evidence):

```
process_events  trace_rows  trace_newlines  identity_total
          350          64              64             414

$ glake stats datalake/raw-local        # 004 reference binary, same instant
6 files · 414 events
```

**The identity holds exactly: 350 + 64 = 414**, and trace_rows =
trace_newlines (every trace line is one JSON object; zero malformed/blank
lines in the live lake, so glake's event total needed no malformed term).
Instant matters: the hooks appended 2 events to dt=2026-07-10 *during
this session* (90 lines at session start, 92 at measurement) — the
identity is only meaningful when both sides are read back-to-back, which
is itself the R10 lesson resurfacing. The `s3/` variants are the same
generated SQL over `s3://goldeneye-lake/raw/dt=*/*.json*` (+ traces path)
with the httpfs preamble; NOT runnable here (no credentials) — first live
run is sitting U, where the identity gains its "+ events ingested since
the sync" term (stated in the query headers).

## 6. Spec frictions (for the parent session / doc revision)

1. **Requirements' dry-run transcript** shows `plan: upload 7 (301
   events), skip 0` — the `(NNN events)` decoration implies the sync
   parses lake content, which the design's "sync moves bytes, not
   judgments" line contradicts. Implemented without it (decision 7);
   either drop the decoration from the transcript at re-gate or add an
   explicit requirement line (it would cost a glake-style classify pass
   inside plan).
2. **S2's parenthetical** "(fake's ledger shows none)" can't literally
   apply to the *binary's* dry-run, which has no store at all (that's the
   feature); the reference satisfies it at lib level and proves the
   binary's independence with a scrubbed environment. Suggest S2 note the
   proof is structural for the CLI.
3. **"~60 lines" for S3Store** reads as a file cap; the module is ~120
   lines because the mandated teaching comments (quote-strip, etag
   caveats, pagination) live there. Code-only it is ~60. Wording only.
4. **S5's "unreadable path"**: a chmod-000 test is impossible in this
   environment (tests run as root, which ignores mode bits); covered with
   a nonexistent path (same GlakeError→exit-2 path). Sitting materials on
   the learner's machine can do the chmod variant.
5. **006 H1/H7 change-protocol amendments are deliberately NOT logged by
   this reference** — S6b assigns them to sitting T ("until then 006
   stands as approved for its own sittings, which run first"). The
   supersession is implemented and commented in code; the 006 changelog
   entries remain for the parent session at sitting T.
6. **aws-config's dependency fan-out**: `cargo tree | grep -c aws-sdk` =
   4 (sso, ssooidc, sts ride with aws-config as credential providers).
   Any future "which SDKs are in the tree" assertion should expect the
   family, not just aws-sdk-s3.
7. **cdk-nag's suggested acknowledgment strings are unusable as printed**
   for BOTH finding shapes this spec hit (granular-with-`::` and
   pack-qualified) — 006's upstream-issue candidate got two more data
   points (§3).
8. **The lake is a moving target during authoring sessions** (hooks
   append live). Any recorded count in evidence.md needs its instant;
   suggested: the sitting-U evidence template carries a timestamp column
   for each identity term.

## 7. Housekeeping

This session created build outputs only under the references' `target/`
dirs (git-ignored) and `infra/cdk.out` + `node_modules` (infra
.gitignore). The 006 reference crate was **not modified** (verified via
git status); its bootstrap remains on disk for the baseline row. An
automated WIP snapshot commit (`9891b76 007: reference WIP snapshot`)
landed mid-session from the environment's checkpointing; this session
itself committed nothing, per its brief — final-state commits are the
parent session's call.

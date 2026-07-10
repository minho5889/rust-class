# 006 hello-lambda — reference implementation notes

Validation record for `_reference/hello-lambda/` (the answer key) and the
real `infra/` CDK app, authored 2026-07-10. Everything below ran in this
session; exit codes were captured directly (no pipe-masking).

---

## 1. The H6 numbers (the artifact)

Built with the REAL toolchain — no fallback needed:
`cargo-lambda 1.9.1` (at `/usr/local/bin`) + Zig via the `ziglang` Python
package (cargo-lambda's zig discovery found it automatically).

```console
$ cargo lambda build --release --arm64        # exit 0
$ ls -l target/lambda/hello-lambda/bootstrap
-rwxr-xr-x 1 root root 1843256 ... bootstrap
$ file target/lambda/hello-lambda/bootstrap
ELF 64-bit LSB pie executable, ARM aarch64, dynamically linked,
interpreter /lib/ld-linux-aarch64.so.1, for GNU/Linux 2.0.0, stripped
```

- **Bootstrap size: 1,843,256 bytes = 1.76 MiB** (single-digit MB as H6
  expected — this is the no-SDK baseline 007 measures against).
- Section breakdown of the actual ARM64 artifact
  (`aarch64-linux-gnu-size`): text 1,767,224 · data 72,984 · bss 5,208.
- Built under the replicated `[profile.release]` (lto=thin,
  codegen-units=1, panic=abort, strip=symbols) — replicated verbatim from
  the workspace root into the reference's own manifest, with a comment
  saying why (the reference lives outside the workspace; same flags or the
  size number lies about what `crates/hello-lambda` will get).
- "Dynamically linked" is expected and fine: cargo-lambda's zig build
  links against glibc-compatible symbols present in provided.al2023; the
  binary carries no runtime, no GC, no JIT — the cold-start argument (T3)
  is about those, not about static-vs-dynamic libc.

### cargo bloat (host x86-64 — bloat cannot cross-compile; approximation, as design.md notes)

Top crates (`cargo bloat --release --crates -n 10`, host .text 1.3 MiB):

| Crate | .text share | Size |
|---|---|---|
| std | 39.8% | 519.9 KiB |
| serde_json | 6.4% | 83.7 KiB |
| tokio | 6.2% | 81.4 KiB |
| hyper_util | 6.1% | 80.0 KiB |
| http | 5.5% | 71.9 KiB |
| hyper | 4.9% | 63.9 KiB |
| lambda_runtime | 3.7% | 48.9 KiB |
| url | 3.2% | 42.1 KiB |
| encoding_rs | 2.7% | 34.7 KiB |
| idna | 2.2% | 29.1 KiB |

Top single function: `encoding_rs::...::decode_to_utf8_raw` (26.8 KiB);
then hyper_util's client send_request and lambda_runtime's
process_invocation closure (~22 KiB each). Reading for the learner: the
binary is mostly *the HTTP client that talks to the Lambda Runtime API*
(hyper/http/url/encoding chain pulled in by lambda_runtime) plus std —
glake and the handler are noise-level. That's what "your binary IS the
runtime" costs, and it's < 2 MiB total.

## 2. Crate validation (all exit codes captured with `; echo exit=$?`)

| Check | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 (unwrap_used/expect_used denied crate-wide) |
| `cargo test` | exit 0 — **10 tests, all green** |
| `cargo tree -e normal \| grep -c aws-sdk` | 0 (H7: only lambda_http/lambda_runtime family + tokio/serde_json/tracing/glake) |

Test census: 4 unit (door fixed-order + multiline re-serialization; clock
math against known instants incl. a real lake event_id prefix; OnceLock
once-ness) · 5 examples in `tests/handler.rs` (H1 ×2, H2, H3 healthz, H3
404-fallthrough — all driving `handle_with` with hand-built lambda_http
Requests, H5) · 1 property `prop_h4_door_equivalence` in
`tests/prop_door.rs` — **512 cases** (design floor 256), each case a mixed
batch, so a run compares a few thousand verdict pairs against the live
relay router. No counterexamples found (no `proptest-regressions/`
directory exists — nothing to commit).

## 3. infra/ synth transcript summary

Toolchain: aws-cdk-lib **2.261.0**, CDK CLI 2.1130.0, **cdk-nag 3.0.1**,
cargo-lambda-cdk 0.0.32, node 22.22.2, ts-node 10.9.2, TypeScript 5.9.3.

```console
$ npm install                                        # exit 0
$ npx tsc --noEmit                                   # exit 0
$ npx cdk synth -c helloLambdaDir=../specs/006-hello-lambda/_reference/hello-lambda
#                                                    # exit 0, zero findings
$ npx cdk synth                                      # exit 1 (EXPECTED - see below)
```

Default-context synth fails with
`'/home/user/rust-class/crates/hello-lambda/Cargo.toml' is not a path to a
Cargo.toml file` until the learner's crate exists (sitting O). Documented
in infra/README.md; reference mode is the authoring/CI path.

Template assertions (read from `cdk.out/goldeneye-stateless.template.json`):

- `AWS::Lambda::Function` → `Architectures: ["arm64"]`,
  `Runtime: provided.al2023`, `MemorySize: 128`, `Timeout: 10`,
  `Tags: [{project: goldeneye}]` ✓
- `AWS::Lambda::Url` → `AuthType: NONE` ✓ — with **two**
  `AWS::Lambda::Permission` resources: the `lambda:InvokeFunctionUrl` one
  (`FunctionUrlAuthType: NONE`) and a second `lambda:InvokeFunction`
  permission scoped by `InvokedViaFunctionUrl: true` (a favorable,
  scoped-down detail — sitting P reads both)
- `AWS::IAM::Role` → only `AWSLambdaBasicExecutionRole` managed policy,
  project tag ✓
- Bundled asset in cdk.out is byte-identical in size (1,843,256) and
  `file`-verified aarch64 — cargo-lambda-cdk ran the same local
  cargo-lambda build.
- `validation-report.json`: `pluginReports: []` — zero unacknowledged
  findings across BOTH packs.

### cdk-nag: what actually fired (discovery run, acknowledgments disabled)

| Rule ID (as reported) | Level | Resource |
|---|---|---|
| `AwsSolutions-IAM4[Policy::arn:<AWS::Partition>:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole]` | ERROR | HelloLambda/ServiceRole |
| `Serverless-LambdaDLQ` | ERROR | HelloLambda function |
| `Serverless-LambdaTracing` | WARNING | HelloLambda function |

All three are acknowledged in `lib/stateless-stack.ts`, each with a written
reason (managed policy = exactly the logs-only access needed; synchronous
Function-URL invocation makes a DLQ unreachable; H10 reads REPORT lines,
not X-Ray traces).

**Finding: neither pack has a Function-URL-auth rule.** The
AuthType=NONE decision therefore carries NO cdk-nag suppression — there is
nothing to suppress in cdk-nag 3.0.1 (AwsSolutions + Serverless). Per the
audit directive, no suppression was invented; the written justification
(risk named, bounded, expiried) lives as the block comment above the URL in
the stack file and in infra/README.md. If a future cdk-nag grows such a
rule, synth fails and forces the text into a real acknowledgment.

**cdk-nag v3 API notes (recorded as required):**

- v3 wires through CDK-native `Validations.of(app).addPlugins(new
  AwsSolutionsChecks(app), new ServerlessChecks(app))` — the research doc
  was right that Aspects-based v2 snippets are stale. Suppressions are
  `Validations.of(construct).acknowledge({id, reason})`; acknowledgment
  propagates to descendant constructs (verified in cdk-nag source:
  `isAcknowledged` walks ancestor scopes).
- **Rough edge found:** the granular `AwsSolutions-IAM4[Policy::…]` id
  embeds `::`, and aws-cdk-lib 2.261's `Validations.acknowledge()` rejects
  any id with more than one `::` pair (`qualifyId` reserves the delimiter)
  — even though the synth output literally suggests acknowledging that
  exact string. Also, the CLI hint's `Pack::RuleId` form does not match
  cdk-nag 3.0.1's own matcher (it compares the bare rule id after
  stripping only an `annotation::` prefix); the plain base id
  (`AwsSolutions-IAM4`) doesn't match a granular finding either — both
  were tried and both left the error standing. Working resolution: record
  that ONE acknowledgment via the public
  `Validations.ACKNOWLEDGED_RULES_METADATA_KEY` metadata key — the same
  channel `acknowledge()` writes and cdk-nag reads. Documented in the
  stack file; worth an upstream issue.
- cargo-lambda-cdk quirk: a manifest containing any `[workspace]` table —
  including the reference crate's empty opt-out — is treated as a
  workspace and demands `binaryName`. Set to `hello-lambda` (harmless for
  the learner's crate too).

## 4. Semantic decisions beyond the spec text (these feed the gate)

1. **Crate shape: lib + thin bin** (`lib.rs` exporting `handler`/`state`,
   `main.rs` = subscriber + eager state touch + `run(service_fn(...))`),
   mirroring relay — required so `tests/` can drive the handler in-process
   (H5) without a network.
2. **The emit seam is a two-line trait** (`Emit` / `StdoutEmit`), the
   handler's only injection point; production `println!` lives in
   `StdoutEmit::emit` at the outermost edge. Cost: one dyn dispatch per
   *accepted* event, nothing on reject/healthz paths. (This is also 007's
   swap point — an S3-backed sink replaces `StdoutEmit` without touching
   the door or routing.)
3. **The door is re-derived from glake, not imported from relay:** relay
   is a dev-dependency only (the H4 oracle). The three error strings are
   duplicated VERBATIM from relay's `Rejection` Display texts; the H4
   property is the tripwire if either side ever rewords one. (Sharing a
   door crate was rejected: 006's lesson is that the door survives the
   platform move *because both callers use the same glake API*, and H4
   exists precisely to prove non-drift instead of assuming it.)
4. **404-only fallthrough, no 405:** H3 says "any other method/path
   returns 404", so `GET /events` → 404 from hello-lambda while relay's
   axum answers 405 (verified empirically). Divergence is OUTSIDE H4's
   domain (the generator only speaks `POST /events`) and outside H1–H3's
   letter; noted here because a doc reader comparing the two servers by
   hand will find it.
5. **Non-UTF-8 request bodies fold into "not a json object"** (they can't
   be JSON objects). Relay/axum would answer such bodies with axum's own
   UTF-8 rejection text instead. Outside the property domain (proptest
   generates Rust `String`s = valid UTF-8 by construction); documented
   rather than reconciled.
6. **Instance identity (H7 rev 2 amendment applied):**
   `$AWS_LAMBDA_LOG_STREAM_NAME` verbatim when present (unique per
   instance, free, links healthz answers to CloudWatch streams), else
   `i-<16 hex>` from init-nanos ⊕ pid through one SplitMix64 round — no
   `rand` dep. `started` is computed by a hand-rolled, unit-pinned
   Hinnant civil-from-days ("rfc3339-ish", no leap seconds — exactly as
   honest as the hooks' own `ts`); a date crate was not worth binary
   weight for one string formatted once per cold start.
7. **State init is EAGER** — `main` touches `state::instance()` before
   `lambda_http::run` and logs a `cold start` line to stderr, so `started`
   means cold-start time, never first-request time (and the stderr line
   doubles as an H10 cold-start marker).
8. **Test-side global-state discipline:** the counters are process-global
   on purpose (T4), so `tests/handler.rs` serializes handler-driving tests
   with a `tokio::sync::Mutex` (std Mutex across `.await` is exactly what
   clippy::await_holding_lock exists to catch) and asserts H3 as
   before/after DELTAS. The awkwardness is called out in the test docs as
   the T4 lesson leaking into the harness.
9. **H4 harness:** relay runs router-only with the channel receiver held
   alive (its 202s require a live receiver — precondition asserted:
   batch < capacity 256, receiver held to scenario end), current-thread
   runtime (equivalence is per-request; nothing needs to race), 512 cases
   of mixed batches with exact duplicates. The property additionally
   pins emit discipline: exactly one emitted line iff 202. Bodies are
   asserted ≤ 64 KiB (the amended domain bound); the >limit divergence
   (axum stock 413 vs Lambda's platform-level ~6 MB rejection) is
   documented in the test's module docs as outside the domain.
10. **lambda_http feature-picked** (`default-features = false`, only
    `apigw_http` + `tracing`): Function-URL payloads are API-GW-v2-shaped;
    the ALB/REST/WebSocket/VPC-Lattice codecs never arrive. Same
    name-what-you-use discipline as relay's tokio block; tokio itself
    shrank from relay's seven features to two (the T1 lesson in
    Cargo.toml form).
11. **infra:** memorySize 128 (H10's lower rung; the experiment raises it
    to 512 via console/CLI or a one-line edit), timeout 10 s, two
    CfnOutputs (FunctionUrl for H9, FunctionName for fetching REPORT
    lines in H10). `helloLambdaDir` context key defaults to
    `../crates/hello-lambda`; the stateful stack file exists as a
    documented stub and is NOT instantiated (007 adds the `new
    StatefulStack(...)` line).
12. **`Serverless-LambdaTracing` (warning-level) is acknowledged too**, with
    a reason, so the nag report is empty rather than "empty except noise" —
    reviewers should see zero findings or real ones.

## 5. Spec frictions (for the parent session / doc revision)

- ~~**requirements.md's healthz example** shows `"instance":"i-3f9a…"`~~ —
  **resolved**: requirements rev 2.1 shows the log-stream shape.
- ~~**design.md's state row** says "init timestamp + a few random bytes"~~ —
  **resolved**: design rev 2 says env-var-first, nanos+pid fallback.
- ~~**design.md's auth decision** says "NONE + cdk-nag suppression"~~ —
  **resolved**: the decision row now records the rule-absence finding
  (cdk-nag 3.0.1 has no Function-URL-auth rule in either pack) and points
  at H12's either/or.
- **H10 note (doc-level):** the cold-start table now also records **Max
  Memory Used** from the REPORT lines — the sitting-Q materials should
  include that column. No code impact on this reference.
- Housekeeping for the parent session: this session created
  `_reference/hello-lambda/target/` and `infra/node_modules/`+`cdk.out/`
  build outputs. `target/` is covered by the repo root `.gitignore`
  (`target/`); infra has its own `.gitignore` for the other two. Nothing
  was committed from this session.

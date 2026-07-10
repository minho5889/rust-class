# Design — 006 hello-lambda

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

Everything you built in 005 that the platform now owns, crossed out:

```
 ~~listener~~  ~~router loop~~  ~~graceful shutdown~~  ~~single-writer task~~
 curl ─► Function URL ─► Lambda service ─► YOUR async fn ─► stdout line ─► CloudWatch
                          (spawns/kills instances,          (the lake in exile;
                           one request per instance          S3 home arrives in 007)
                           at a time, scales by adding
                           instances)
```

One file of Rust survives the move: the **door**. `validate.rs` logic from
relay is reached the same way relay reached it — through your glake lib —
so 202/400 behavior is identical by construction, and H4 makes "identical"
a proven property instead of a hope.

The concurrency model flips, and that's T4: relay was *one process, many
tasks, shared counters*; Lambda is *many instances, one request each, nothing
shared*. Your healthz counters still compile, still work — and now mean
something completely different. We keep them **on purpose**, to watch that
meaning change.

## The shape

| Part | What it is | Rust you learn | REQs |
|---|---|---|---|
| **crates/hello-lambda/src/main.rs** | `#[tokio::main]`: install `tracing_subscriber` **writing to stderr** (stdout is the event-line channel — both land in the same CloudWatch stream, but event lines stay grep-ably pure), **eagerly initialize state** (so `started` = cold-start time, not first-request time), then `lambda_http::run(service_fn(handler))` | the runtime loop you no longer write | H1–H3a, H7 |
| **handler.rs** | `async fn handler(req: Request) -> Result<Response<Body>, Error>` — match (method, path): POST /events → door; GET /healthz → counters; else 404. The door path **returns** `(Response, Option<String>)` — the accepted line comes back as a value (the testable seam, relay's writer-seam lesson recycled); `println!` happens only at the outermost edge | lambda_http Request/Response, the one-fn server, seams for testability | H1–H3b, H5 |
| **door (via glake)** | the same three-clause check relay makes (restated in requirements so this spec stands alone): glake classification + comparable day → 202/400 in the fixed first-problem order | your lib's third caller | H1, H2, H4 |
| **state.rs** | `OnceLock<Instance>` set eagerly in `main` — instance id from `AWS_LAMBDA_LOG_STREAM_NAME` when present (unique per instance, free), else init-time nanos + process id (local runs); started-at; three `AtomicU64`s | per-instance state, said honestly | H3a |
| **infra/** | CDK v2 TS app: `bin/goldeneye.ts`, `lib/stateless-stack.ts` (RustFunction ARM_64 + FunctionUrl), `lib/stateful-stack.ts` (documented stub, 007), **cdk-nag v3 with both the AwsSolutions and Serverless packs** (wired per the current cdk-nag API — verified against the installed major version, not from memory), `Tags.of(app).add("project","goldeneye")`. Suppressions name **real rule IDs discovered from actual nag output**; if no rule covers Function-URL auth, that absence is recorded, not papered over with an invented ID | — (Claude writes TS; learner reviews as the AWS pro) | H8, H12 |

## Key decisions

| Decision | Options | Chosen | Why |
|---|---|---|---|
| HTTP layer | raw `lambda_runtime` + hand-rolled event types · **`lambda_http`** | lambda_http | Function URL payloads are API-GW-v2-shaped; lambda_http gives real `http::Request`/`Response` types — the 005 mental model carries over exactly; raw runtime shown in one sitting-O aside |
| Where events go | /tmp file · ignore · **one JSON line on stdout → CloudWatch** | stdout | a Lambda's disk is a cache, not a lake; stdout is the only durable-by-default sink it has. Makes "compute vs storage" impossible to miss, and makes 007 *wanted* |
| Function URL auth | AWS_IAM (+SigV4 curl gymnastics) · **NONE + cdk-nag suppression + same-sitting teardown** | NONE, suppressed in writing | it's a lab endpoint alive for one sitting, receiving only telemetry-shaped JSON, writing only to its own logs. The suppression text IS the security lesson: name the risk, bound it, expiry it. IAM variant documented in the stack file for anyone keeping it up |
| Counters | drop them · **keep relay's counters verbatim** | keep | same code, new semantics = the sharpest possible T4 lesson; healthz shows `instance`+`started` so two curls to a scaled function visibly hit different instances |
| Who writes the TS | learner · **Claude writes, learner reviews** | Claude | TypeScript CDK is the learner's day job, not the learning target; coached mode spends the learner's writing time on Rust (intent assumption, 002 precedent) |
| Reference profile | inherit workspace `[profile.release]` · **replicate it in the reference's own manifest** | replicate | the reference lives outside the workspace; H6's size numbers are only honest if the profile matches what `crates/hello-lambda` will get from the workspace root |

## Properties (co-written, test-first)

| REQ | Property | Generation strategy |
|---|---|---|
| H4 | ∀ bodies ≤ 64 KiB: `(status, body_text)` of hello-lambda's door ≡ relay's door | reuse 005's A4 message strategy (valid envelopes over a multi-day domain ∪ malformed arms ∪ duplicates), size-bounded; drive hello-lambda via a `lambda_http::Request` and relay via its router `oneshot` — **with relay's writer receiver alive** (a 202 with a dead channel degrades to 500 and would fail equivalence for the wrong reason; the harness builds the router the way relay's own tests do, temp lake and all). The relay side is a dev-dependency on the 005 reference lib. Compare status + body. ≥256 cases |

One property is the right number here: 006's risk is not logic (the door is
imported, not rewritten) — it's **drift** between the two deployment shapes.
H4 pins exactly that. The heavy lifting moved to [O] measurements.

## How we verify

**In-session (the reference is validated with all of these before this gate
is presented — if you're reading this at the gate, it exists and passed):**
- H4 proptest ≥256, written before the handler wiring (red against a stub).
- H1–H3/H5 as `#[tokio::test]` with hand-built `lambda_http` Requests.
- fmt + clippy (`unwrap_used`, `expect_used` deny) clean.
- `cargo lambda build --release --arm64` (real cargo-lambda + zig, installed
  in-session): artifact exists, size recorded, `file` says aarch64.
- `cargo bloat --release -n 10` recorded (host target — bloat can't cross-
  compile; noted as approximation).
- `cd infra && npx cdk synth` with cdk-nag (both packs): zero unsuppressed
  findings; template asserts: ARM_64, PROVIDED_AL2023, FunctionUrl
  AuthType=NONE with the written suppression (or the recorded rule-absence
  note), project tag present. Synth bundling requires cargo-lambda locally —
  stated as sitting-P's prerequisite.

**Deploy day (sitting Q → evidence.md):** H9 transcripts, H10 cold-start
table (≥5 × {128, 512} MB), H11 teardown + cost note.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-10 | Initial fast-path draft | Part-5 directive (full loop) | pending combined ack |
| 2026-07-10 | Rev 2 per design+tasks audit (68%): emit seam designed in (door returns the line; println at the edge — H1/H2 become testable, MAJOR-2); cdk-nag corrected to v3-current API + both packs + real-rule-ID discipline (MAJOR-3); validation claims re-tensed truthful-at-gate (MAJOR-1); tracing→stderr design element; H4 harness names its relay dev-dep, size bound, and the live-receiver precondition; instance id from log-stream env; eager state init (started = cold start) | 006 audits | this combined gate |

</details>

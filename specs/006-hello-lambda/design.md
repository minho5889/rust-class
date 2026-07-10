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
| **crates/hello-lambda/src/main.rs** | `#[tokio::main]` → `lambda_http::run(service_fn(handler))`; `#[cfg(feature="lens")]` allocator hook kept out (lens is a local tool; noted) | the runtime loop you no longer write | H1–H3 |
| **handler.rs** | `async fn handler(req: Request) -> Result<Response<Body>, Error>` — match (method, path): POST /events → door; GET /healthz → counters; else 404. Accepted → `println!("{line}")` (one compact JSON line, stdout = CloudWatch) | lambda_http Request/Response, the one-fn server | H1–H3, H5 |
| **door (via glake)** | same call relay makes: classification verdict + first-missing-key + comparable day → 202/400 in 005's fixed order | your lib's third caller | H1, H2, H4 |
| **state.rs** | `OnceLock<Instance>` — instance id (from init timestamp + a few random bytes), started-at, three `AtomicU64`s | per-instance state, said honestly | H3 |
| **infra/** | CDK v2 TS app: `bin/goldeneye.ts`, `lib/stateless-stack.ts` (RustFunction ARM_64 + FunctionUrl), `lib/stateful-stack.ts` (documented stub, 007), cdk-nag `AwsSolutions` aspect, `Tags.of(app).add("project","goldeneye")` | — (Claude writes TS; learner reviews as the AWS pro) | H8 |

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
| H4 | ∀ bodies: `(status, error_text)` of hello-lambda's door ≡ relay's door | reuse 005's A4 message strategy (valid envelopes over a multi-day domain ∪ malformed arms ∪ duplicates); drive hello-lambda via a `lambda_http::Request` and relay via its router `oneshot`; compare status + body. ≥256 cases |

One property is the right number here: 006's risk is not logic (the door is
imported, not rewritten) — it's **drift** between the two deployment shapes.
H4 pins exactly that. The heavy lifting moved to [O] measurements.

## How we verify

**In-session (the reference was validated with all of these):**
- H4 proptest ≥256, written before the handler wiring (red against a stub).
- H1–H3/H5 as `#[tokio::test]` with hand-built `lambda_http` Requests.
- fmt + clippy (`unwrap_used`, `expect_used` deny) clean.
- `cargo lambda build --release --arm64` (real cargo-lambda + zig, installed
  in-session): artifact exists, size recorded, `file` says aarch64.
- `cargo bloat --release -n 10` recorded (host target — bloat can't cross-
  compile; noted as approximation).
- `cd infra && npx cdk synth` with cdk-nag: zero unsuppressed findings;
  template asserts: ARM_64, PROVIDED_AL2023, FunctionUrl AuthType=NONE with
  the written suppression, project tag present.

**Deploy day (sitting Q → evidence.md):** H9 transcripts, H10 cold-start
table (≥5 × {128, 512} MB), H11 teardown + cost note.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-10 | Initial fast-path draft | Part-5 directive (full loop) | pending combined ack |

</details>

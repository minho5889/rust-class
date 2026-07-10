# Requirements — 006 hello-lambda (the door goes to the cloud)

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

Your relay's door — the exact same 202/400 validation you built in 005 —
becomes an **AWS Lambda function** behind a Function URL, on ARM64/Graviton:

```console
$ cargo lambda build --release --arm64          # one real deployable artifact
$ ls -lh target/lambda/hello-lambda/bootstrap   # a few MB, statically honest

$ cd infra && npx cdk deploy goldeneye-stateless
 ✅  goldeneye-stateless  (FunctionUrl = https://xxxx.lambda-url.us-east-1.on.aws/)

$ curl -si -X POST "$URL/events" -d @one-event.json | head -1
HTTP/1.1 202 Accepted

$ curl -s -X POST "$URL/events" -d '{"not":"an envelope"}'
{"error":"missing key: event_id"}

$ curl -s "$URL/healthz"
{"instance":"i-3f9a…","started":"2026-07-12T…","received":3,"accepted":2,"rejected":1}
```

Accepted events don't hit a disk — a Lambda has none worth keeping. Each one
is printed as **one JSON line to stdout**, which CloudWatch Logs captures:
the lake entry in exile. (007 gives events their real S3 home; this spec is
about the compute, not the storage.)

Why this is the lesson: in 005 you wrote a whole server — listener, router,
graceful shutdown, single-writer. On Lambda, **the platform owns all of
that**. What's left of your program is one async function. That's why Rust
fits so well here: no JVM to warm, no GC to size — your binary *is* the
runtime (`provided.al2023` just runs `bootstrap`), so cold start is mostly
"how fast does Linux exec a few-MB static-ish binary." This spec makes you
*measure* that claim instead of repeating it.

**What runs where (honesty box):** this spec was authored in a session with
no AWS credentials. Everything below tagged *(local)* was validated during
authoring — tests, the property, the real ARM64 artifact, `cdk synth` +
cdk-nag. Lines tagged *(deploy day)* run in sitting Q on **your** account,
with Claude co-driving, and land in `evidence.md`.

## What we're *not* building yet

- **Any S3 / real lake writes** → 007. No aws-sdk-* crates in this binary —
  that's deliberate; 007 measures exactly what the SDK costs.
- Other compute targets (MicroVMs, Fargate, EC2) → later specs.
- Auth on the Function URL beyond the lab decision in design.md; custom
  domains; API Gateway. This is a lab endpoint, deployed and torn down in
  one sitting.

## What you'll learn building it

- **T1 · The Lambda model** — your whole 005 server collapses into one
  `async fn(Request) -> Response`; the platform owns listening, concurrency,
  and shutdown.
- **T2 · provided.al2023 + bootstrap** — what "custom runtime" really means;
  why Rust ships as a bare executable.
- **T3 · The release profile in anger** — lto/codegen-units/panic=abort/strip
  finally justify themselves in artifact size and cold start.
- **T4 · Per-instance state** — your healthz counters silently become
  per-instance; what that implies for every "just keep it in memory" plan.
- **T5 · The measured claim** — cold start and memory on Graviton, from your
  own CloudWatch REPORT lines, versus what a GC runtime would need.

---

## Precise acceptance criteria

> Tags: **[P]** property · **[E]** example · **[O]** operational.
> Scope tags: *(local)* validated in-session · *(deploy day)* sitting Q,
> learner's account, results → evidence.md.

**The handler** *(all local)*
- **[E] H1** — `POST /events` with a valid body (005's door check, verbatim:
  glake classification + comparable day) returns **202** with an empty body,
  and emits the accepted event as **exactly one compact JSON line on stdout**.
- **[E] H2** — `POST /events` with an invalid body returns **400** with the
  same one-line JSON error, in the same fixed first-problem order, as relay
  (005 A2); nothing is emitted to stdout for rejected bodies.
- **[E] H3** — `GET /healthz` returns **200** with per-instance counters
  `{instance, started, received, accepted, rejected}`; any other
  method/path returns 404. The doc and code must say out loud: these
  counters are **per warm instance** — they reset on cold start and are not
  shared across concurrent instances (T4).
- **[P] H4** — door equivalence: for any generated body (005's A4 message
  strategy), hello-lambda's (status, error text) equals relay's. The two
  doors may never drift.
- **[E] H5** — all of H1–H3 are proven by local `#[tokio::test]`s driving the
  handler with `lambda_http` Request values — no AWS, no network.

**The artifact** *(local)*
- **[O] H6** — `cargo lambda build --release --arm64` produces
  `target/lambda/hello-lambda/bootstrap`; its size and the `cargo bloat`
  top-10 are recorded (expected: single-digit MB — the no-SDK baseline 007
  will be measured against).
- **[O] H7** — dependency policy: `lambda_http`/`lambda_runtime`, `tokio`,
  `serde_json`, `tracing` (+subscriber), and the glake lib (path dep). **No
  `aws-sdk-*`.** `clippy::unwrap_used` + `expect_used` denied (deployable
  crate).

**The infrastructure** *(local synth; deploy is Q)*
- **[O] H8** — `infra/` is born: a CDK v2 TypeScript app with a
  **stateless stack** containing the function (cargo-lambda-cdk
  `RustFunction`, explicit `Architecture.ARM_64`) and its Function URL;
  every resource tagged `project=goldeneye`; `cdk synth` passes with
  **cdk-nag** (AwsSolutions) clean or carrying written suppressions; the
  stateful stack file exists as a documented stub for 007.

**Deploy day** *(all deploy day)*
- **[O] H9** — deployed to us-east-1 on the learner's account; the three
  curl transcripts above reproduced against the real URL; the stdout event
  line found in CloudWatch Logs.
- **[O] H10** — the cold-start experiment: ≥5 cold starts each at 128 MB and
  512 MB; `REPORT`-line init + duration numbers tabulated in evidence.md,
  with one paragraph comparing against a GC-runtime baseline from the
  research reports (cite, don't hand-wave).
- **[O] H11** — teardown the same sitting: stateless stack destroyed;
  `evidence.md` records the deploy/destroy timestamps and (approximate) cost.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-10 | Initial fast-path draft (with design+tasks); Parts 3–5 audit lessons pre-applied: single-clause EARS, scope tags for the no-credentials reality, [P] strategy lives in design, door defined by reference to 005 rather than re-specified | Part-5 directive (full loop) | pending combined ack |

</details>

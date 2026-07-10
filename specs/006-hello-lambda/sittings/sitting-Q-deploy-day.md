# Sitting Q — deploy day

**Builds:** nothing new — this sitting *spends* what O and P built: the
stack goes up on **your** account (Claude co-drives), the three H9 curl
transcripts happen against a real `lambda-url.us-east-1.on.aws` URL, your
event line surfaces in CloudWatch, the cold-start experiment (H10) turns
"Rust cold starts fast" from a slogan into a table with your name on it,
and the stack comes down the same sitting (H11). Evidence.md gets all of it.
**Requirements:** H9–H11 — task 1.5 (T5: the measured claim).
**Ramp you'll use:** sitting N's live-ritual discipline (transcripts and
numbers into evidence, not vibes) and sitting P's template read — you
already know every resource that's about to exist.

> **Honesty box (requirements.md):** this spec was authored with no AWS
> credentials, so *nothing on this page could be validated in advance*.
> Commands are real and were desk-checked against the validated template
> and the platform's documented behavior; outputs are marked **(expected
> shape)** and your real ones replace them in `evidence.md`. Where a number
> below is a prediction, it says so and cites its source. Local commands
> (git, evidence) keep the usual verbatim treatment.

## Where you are

The artifact is 1.76 MiB of aarch64 you can account for crate by crate; the
template is six resources you personally reviewed, down to the two scoped
permissions; the NONE posture carries your written acknowledgment. What's
left is the part the course can't do for you: an account, a region, and the
discipline of measuring instead of believing. You've run hundreds of deploys
more complicated than this one — the point of today isn't "learn to deploy",
it's *what you measure once it's up*: this is the first time in your career
the thing cold-starting is a binary whose every byte you audited two
sittings ago. Claude co-drives: reads outputs with you, drafts the evidence
entries, and watches the clock on H11's promise. You own the credentials and
every decision that costs money.

## The build, move by move

All commands from the repo root unless noted. One commit at the end:
evidence. Approximate cost of everything below, stated up front (AI
proposes, human disposes): a few dozen Lambda invocations at 128–512 MB for
sub-second durations plus a few MB of CloudWatch ingestion — **fractions of
a cent, likely $0.00 even without free tier**; the only way to spend real
money today is to *forget the teardown*, which is why teardown is an
acceptance criterion (H11) and not a chore.

1. **Pre-flight (checklist — all boxes before any deploy).**

   ```console
   aws sts get-caller-identity          # the RIGHT account? your personal lab one
   aws configure get region             # us-east-1 (or pass --region on every call)
   cd infra && npx cdk bootstrap        # once per account/region ever; a no-op if done
   date -u +%FT%TZ                      # ← evidence.md: deploy-day start timestamp
   ```

   Also on the checklist: sitting P's synth still passes (`npx cdk synth`,
   exit 0 — if you touched the stack during review, re-gate your own
   edit); `evidence.md` is open in a buffer, because today's rule is
   **transcripts land as they happen**, not from memory afterwards; and
   say the H11 promise out loud — *the stack dies before this sitting
   ends* — because every bound in your H12 acknowledgment leans on it.

2. **Deploy.**

   ```console
   cd infra && npx cdk deploy goldeneye-stateless
   ```

   *(expected shape)* — one IAM-change confirmation prompt (the role +
   the two permissions you already read in the template; approve them as
   the things you reviewed, not as noise), a minute or two of
   CloudFormation, then:

   ```
    ✅  goldeneye-stateless

   Outputs:
   goldeneye-stateless.FunctionUrl = https://<id>.lambda-url.us-east-1.on.aws/
   goldeneye-stateless.FunctionName = goldeneye-stateless-HelloLambda<hash>
   ```

   Capture both into shell vars — the rest of the sitting speaks through
   them:

   ```console
   URL=$(aws cloudformation describe-stacks --stack-name goldeneye-stateless \
         --query "Stacks[0].Outputs[?OutputKey=='FunctionUrl'].OutputValue" --output text)
   FN=$(aws cloudformation describe-stacks --stack-name goldeneye-stateless \
        --query "Stacks[0].Outputs[?OutputKey=='FunctionName'].OutputValue" --output text)
   ```

   One character to respect before the first curl: the URL output **ends
   with a slash**, and sitting O's match is exact on `/events` — so it's
   `${URL}events`, never `$URL/events` (the double-slash 404 is this
   sitting's first fight; details below). Timestamp the deploy in
   evidence.md.

3. **The three transcripts (H9).** Use a real envelope — pull one line
   out of the live lake into a file, the same move the H1 test made:

   ```console
   head -1 datalake/raw-local/dt=2026-07-09/events.jsonl > /tmp/one-event.json

   curl -si -X POST "${URL}events" -H 'content-type: application/json' \
        -d @/tmp/one-event.json | head -1
   # (expected shape) → HTTP/1.1 202 Accepted

   curl -s -X POST "${URL}events" -H 'content-type: application/json' \
        -d '{"not":"an envelope"}'
   # (expected shape) → {"error":"missing key: event_id"}

   curl -s "${URL}healthz"
   # (expected shape) →
   # {"instance":"2026/07/12/[$LATEST]8c3f…","started":"2026-07-12T…:…:…Z",
   #  "received":2,"accepted":1,"rejected":1}
   ```

   Read the healthz body against sitting O's T4 conversation, because it
   just came true: `instance` is a **CloudWatch log-stream name** now —
   the env-var branch of your `state.rs` took the other arm for the first
   time (every local test saw `i-…`), and that string is a *link*: it
   names exactly the stream in move 4 where this instance's lines live.
   `started` is the cold start you just caused; the counters are that one
   instance's short life story. Post the invalid body a couple more times
   and watch `rejected` climb — same instance, same `started`, memory
   holding. Then the T4 live demo, the reason `instance` exists at all:

   ```console
   for i in 1 2 3 4 5 6 7 8; do curl -s "${URL}healthz" & done; wait
   ```

   *(expected shape)* — fired concurrently, the platform's
   one-request-per-instance rule forces scale-out, and the answers should
   show **two or more distinct `instance` values with distinct
   `started` times and disjoint counter histories**. If they all came
   back from one instance (the burst was too quick or too slow), run it
   again — catching the fleet in the act is allowed to take two tries.
   Every "just keep it in memory" plan you've ever seen a customer ship
   fails in exactly the way this output shows. All transcripts →
   evidence.md, verbatim.

4. **Find your lines in CloudWatch (H9 closes).** The log group is
   `/aws/lambda/$FN` — created by the *service* on first invoke, not by
   the stack (sitting P's missing-resource question; it pays off in
   move 7).

   ```console
   aws logs tail "/aws/lambda/$FN" --since 15m --format short
   ```

   *(expected shape)* — three kinds of line, and you can tell whose voice
   each one is:

   ```
   INIT_START Runtime Version: provided:al2023.v… 
   … INFO cold start id="2026/07/12/[$LATEST]8c3f…" started="…"   ← stderr: tracing (main.rs)
   {"event_id":"1783556456860138087-1438-29228","ts":"2026-07-09T00:20:56Z",…}   ← stdout: THE event line
   … DEBUG rejected at the door error="missing key: event_id"     ← stderr: tracing (reject path)
   REPORT RequestId: … Duration: … ms  Billed Duration: … ms  Memory Size: 128 MB
          Max Memory Used: … MB  Init Duration: … ms
   ```

   The middle one is the point of the whole spec: **the accepted event as
   one compact JSON line, in a durable store, emitted by a `println!`** —
   the lake entry in exile, exactly as designed (007 gives it the S3 home;
   grep-ably pure *because* tracing went to stderr, sitting N's discipline
   collecting interest in a different building). Confirm the rejected
   body produced *no* event line (H2, in production). And note the
   platform's own bookkeeping: `INIT_START` → your `cold start` line →
   the first `REPORT` carrying `Init Duration`. Warm invokes get REPORT
   lines with no `Init Duration` — that presence/absence is the
   cold-start detector the experiment leans on.

5. **The cold-start experiment (H10) — run it like an experiment.**
   Hypothesis first, in writing, before any data: predict your median
   init duration at 128 MB, and whether 512 MB will change **init**,
   **duration**, or **Max Memory Used** — and why. (Priors from
   `research/rust-on-aws-compute.md`: ~16 ms init on ARM64 in the
   independent Dec-2025 set; AWS's own reproducible benchmark says
   19–28 ms across memory configs; a light-workload binary showed
   66–68 ms. And remember 128 MB ≈ 0.07 of a vCPU, 512 ≈ 0.29 — CPU
   scales with memory, 1,769 MB ≈ 1 vCPU, so a CPU-light handler's
   *duration* may barely move while anything compute-shaped would.)

   **Forcing a cold start**: any function-configuration change retires
   the warm fleet — the cheapest lever is bumping a throwaway env var.
   The per-sample loop (Claude drives the loop, you read every REPORT):

   ```console
   for i in 1 2 3 4 5; do
     aws lambda update-function-configuration --function-name "$FN" \
       --environment "Variables={COLD_BUMP=$i}" > /dev/null
     aws lambda wait function-updated --function-name "$FN"
     curl -s -o /dev/null -w "%{http_code}\n" -X POST "${URL}events" \
       -H 'content-type: application/json' -d @/tmp/one-event.json
     sleep 3   # let the REPORT line land
   done
   aws logs filter-log-events --log-group-name "/aws/lambda/$FN" \
     --filter-pattern "REPORT" --query "events[].message" --output text \
     | grep "Init Duration"
   ```

   Then the second leg — raise memory and repeat:

   ```console
   aws lambda update-function-configuration --function-name "$FN" --memory-size 512
   aws lambda wait function-updated --function-name "$FN"
   # …the same 5-sample loop…
   ```

   (Two pro-notes, said before you'd say them: the memory change is
   deliberately out-of-band — the stack now *drifts* from its template,
   which is fine only because this stack dies in move 7; the alternative
   is a one-line `memorySize` edit + redeploy if drift offends you. And
   only REPORT lines *with* `Init Duration` count as samples — a warm hit
   in your loop is a discarded row, which is why the env-var bump
   precedes every curl.)

   The table, straight into evidence.md — ≥5 samples per config, plus
   the summary row H10 actually argues from:

   ```markdown
   | # | Memory (MB) | Init duration (ms) | Duration (ms) | Billed (ms) | Max memory used (MB) |
   |---|---|---|---|---|---|
   | 1 | 128 |  |  |  |  |
   | 2 | 128 |  |  |  |  |
   | 3 | 128 |  |  |  |  |
   | 4 | 128 |  |  |  |  |
   | 5 | 128 |  |  |  |  |
   | 1 | 512 |  |  |  |  |
   | 2 | 512 |  |  |  |  |
   | 3 | 512 |  |  |  |  |
   | 4 | 512 |  |  |  |  |
   | 5 | 512 |  |  |  |  |

   median init 128 = … ms · median init 512 = … ms
   memory floor (min Max Memory Used) = … MB at 128 · … MB at 512
   ```

   *(expected shape)* — init in the tens of milliseconds, and **Max
   Memory Used in the low tens of MB regardless of the setting** — that
   floor is the claim's second half (H10 says so explicitly): the 128 MB
   *minimum* configuration is mostly empty around this binary. Compare
   your two `started`-to-first-`REPORT` stories against P's bloat table:
   what you're timing is Linux exec'ing 1.8 MiB and `main` running
   subscriber + `OnceLock` init — there is nothing else *to* warm.

6. **The GC-baseline paragraph (H10 closes — cite, don't hand-wave).**
   One paragraph in evidence.md comparing your table against a
   GC-runtime baseline, citing `research/rust-on-aws-compute.md`
   (verified 3-0 claims unless noted): Rust ARM64 init ~16 ms
   (independent, Dec 2025) and 19–28 ms in AWS's own reproducible
   benchmark, versus **5–8× slower for Python/Node** on the same
   platform — and even the pessimistic light-workload Rust case
   (66–68 ms) stayed 5–6× ahead. The Java comparison is structural, not
   numeric: *"Rust essentially eliminates the problem SnapStart exists
   to solve"* — snapshotting a warmed JVM is a platform feature invented
   because a JVM must warm, and your binary has no equivalent phase to
   snapshot. Add the memory axis, which the research set doesn't cover
   but your table now does: your measured floor of a few tens of MB
   against the hundreds a JVM heap wants *before the first request*, and
   note what that means at the 128 MB price point you just ran at (a
   config a GC runtime can't realistically inhabit). Close with the cost
   line your own numbers now support: ARM64 is 15–20% cheaper per
   GB-second by list price, and cold starts this small make
   per-request billing honest — you paid for work, not for warmup. Your
   sentence, your numbers, the report's citations.

7. **Teardown (H11) — same sitting, verified, and the orphan.**

   ```console
   date -u +%FT%TZ                              # evidence: destroy timestamp
   cd infra && npx cdk destroy goldeneye-stateless
   # (expected shape) → Are you sure…? y → goldeneye-stateless: destroyed

   aws cloudformation describe-stacks --stack-name goldeneye-stateless
   # (expected shape) → …Stack with id goldeneye-stateless does not exist
   curl -s -m 10 "${URL}healthz"; echo "exit=$?"
   # (expected shape) → resolution/connect failure — the subdomain died with the URL resource
   ```

   Now collect on sitting P's question — the resource that was never in
   the template:

   ```console
   aws logs describe-log-groups --log-group-name-prefix "/aws/lambda/$FN" \
     --query "logGroups[].logGroupName"
   # (expected shape) → it's STILL THERE
   ```

   The service created that log group, not CloudFormation — so `cdk
   destroy` doesn't know it exists. Your H12 exposure bound ("a log group
   that dies with the stack") was *approximately* true and precisely
   false; note that in evidence as a review learning — it's exactly the
   class of gap a written justification exists to surface. Your REPORT
   data is already harvested into the table, so:

   ```console
   aws logs delete-log-group --log-group-name "/aws/lambda/$FN"
   ```

   Last, the cost note for evidence.md: invocation count (your healthz
   `received` totals plus the experiment's ~10), GB-seconds (all
   sub-second at ≤0.5 GB), log bytes ingested — and the honest bottom
   line, likely **$0.00, at most cents**. Then the ledger closes:

   ```console
   git add specs/006-hello-lambda/evidence.md
   git commit -m "006: sitting Q — deploy day: H9 transcripts, H10 table, torn down"
   ```

   (Close-out — property-auditor, SKILLS 2a, MEMORY, the
   `spec-close/006-hello-lambda` marker — is task 2.1, Claude's machine
   work, next session.)

## Fights to expect

No compiler today — every fight is a platform fight, and you've probably
diagnosed most of them for other people. Ledger the ones that still bite
(`learning.*`).

- **The double-slash 404.** `$URL/events` against an output that ends in
  `/` sends the path `//events`; sitting O's match is exact, so the
  handler answers 404 and *nothing is wrong anywhere*. Three minutes of
  confusion available for free; `${URL}events` skips them. (Your healthz
  counters won't even move — sitting O pinned that 404s happen before
  the door.)
- **`cdk bootstrap` missing** — first deploy on a fresh account/region
  fails with the SSM-parameter error naming
  `/cdk-bootstrap/hnb659fds/version`. Move 1's checklist line exists
  because of it.
- **Warm rows polluting the experiment** — a REPORT with no
  `Init Duration` is a warm invoke: discard the row, don't average it
  in. Symmetrically: an `update-function-configuration` fired while the
  previous update is still applying returns
  `ResourceConflictException` — that's what the `aws lambda wait
  function-updated` line is for.
- **The base64 wrinkle** — POST without a JSON `content-type` and the
  Function URL may hand lambda_http a base64-marked body; your handler's
  `Body::Binary` arm (written in O, tested never to matter locally)
  quietly earns its keep. Keep the header on your curls anyway so your
  transcripts match the reference shape.
- **Log lines arriving late** — CloudWatch ingestion lags a few seconds;
  a `filter-log-events` fired instantly after a curl can miss the
  REPORT. The loop's `sleep 3` is doing real work.
- **The orphan log group** — the H12 bound said logs die with the stack;
  move 7 shows the one way that's false. If you skip the delete, the
  cost is pennies-per-forever, but the *lesson* costs more: teardown you
  didn't verify isn't teardown.

## Checkpoint

The sitting counts as done only when all of these hold:

```
aws cloudformation describe-stacks --stack-name goldeneye-stateless
                                    # → does not exist (H11 — verified, not assumed)
aws logs describe-log-groups --log-group-name-prefix "/aws/lambda/$FN" \
  --query "logGroups[].logGroupName" # → [] — the orphan is gone too
git log --oneline -1                 # → 006: sitting Q — deploy day: …
```

- `specs/006-hello-lambda/evidence.md` holds, with real values: deploy
  and destroy timestamps; the three H9 transcripts verbatim (including a
  healthz body whose `instance` is a log-stream name); the CloudWatch
  extract showing your stdout event line next to its stderr neighbors;
  the H10 table — ≥5 cold starts × {128, 512} MB with init, duration,
  billed, and Max Memory Used per row, medians and the memory floor
  summarized; the GC-baseline paragraph with its citations; the cost
  note; and the orphan-log-group observation.
- You can answer aloud: how do you *force* a cold start, and how do you
  *detect* one in the logs? Which of the three REPORT numbers did the
  memory setting actually move, and why does 1,769 MB ≈ 1 vCPU explain
  it? What does your measured memory floor say about the smallest
  config this binary can honestly inhabit — and what runtime family
  can't follow it there? Why did two concurrent curls see two
  `started` times (T4, in one sentence)? And the H12 postmortem: which
  written bound turned out approximately-but-not-precisely true, and
  what did it cost to find out?

## Hints (one at a time)

<details><summary>Hint 1 — pulling clean REPORT rows out of CloudWatch</summary>

The REPORT line is key-value text, not JSON; awk it rather than eyeball
it:

```console
aws logs filter-log-events --log-group-name "/aws/lambda/$FN" \
  --filter-pattern "REPORT" --query "events[].message" --output text |
awk '/Init Duration/ {
  for (i=1;i<=NF;i++) {
    if ($i=="Duration:" && $(i-1)!="Billed" && $(i-1)!="Init") d=$(i+1);
    if ($i=="Init" && $(i+1)=="Duration:") init=$(i+2);
    if ($i=="Size:") mem=$(i+1);
    if ($i=="Used:") used=$(i+1);
  }
  print mem" MB  init="init" ms  dur="d" ms  used="used" MB"
}'
```

*(expected shape)* — one line per cold start, ready to transcribe into
the table. If you get fewer lines than samples, either a row went warm
(no `Init Duration` — the filter above drops it correctly) or ingestion
is lagging (re-run in ten seconds). CloudWatch Logs Insights does this
more elegantly — `filter @message like /Init Duration/` — and you know
Insights better than this course does; use whichever gets numbers into
the table faster.

</details>

<details><summary>Hint 2 — the deploy is up but every curl 403s or times out</summary>

Work outward, cheapest first. (1) The URL: `echo $URL` — did the
CfnOutput query actually populate it, and are you appending without the
double slash? (2) A 403 with `{"Message":"Forbidden"}` from a Function
URL usually means the URL resource or its NONE permission didn't
deploy — `aws lambda get-function-url-config --function-name "$FN"`
should show `AuthType: NONE`; if it errors, the stack you deployed isn't
the one you reviewed (stale synth — re-run `npx cdk deploy` and watch
for the permissions in the diff). (3) A timeout with nothing in the
logs: is the region in `$URL` the region you deployed to? (4) If the
function was invoked but errored, the log group has the story —
`aws logs tail` and read the stderr lines; a panic would appear as the
runtime reporting an error *and the instance being replaced on the next
invoke* (watch `instance` change in healthz — the platform demonstrating
why H7 banned unwrap in one log excerpt). None of these cost more than
cents to debug live; the log group is your friend until move 7 deletes
it.

</details>

## If truly stuck

There is no reference deploy to compare against — the honesty box is the
deal — but the paper trail covers everything up to the account boundary:

- `specs/006-hello-lambda/_reference/NOTES.md` §1 and §3 — the exact
  artifact and template that are deploying; if the cloud behaves
  surprisingly, diff reality against these first.
- `infra/README.md` — deploy/destroy commands, the two synth modes, and
  the auth posture summary.
- `research/rust-on-aws-compute.md` — every number move 6 cites, with
  verification votes and the volatility schedule (the cold-start figures
  are marked for annual review — if your table disagrees with the
  research, your table is newer; note it as a research follow-up, don't
  silently trust either).
- `specs/006-hello-lambda/requirements.md` — H9/H10/H11's exact wording,
  which is what evidence.md is answering to.

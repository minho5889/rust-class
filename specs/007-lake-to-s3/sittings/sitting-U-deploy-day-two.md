# Sitting U — deploy day two

**Builds:** nothing new — this sitting *spends* R through T: the stateful
stack goes up on **your** account and you verify its armor by trying to
destroy it (and watching AWS refuse); your real telemetry backlog syncs to
`s3://goldeneye-lake/raw/` with numbers that match the dry-run plan; a
re-run uploads **zero** (the S4 property, collecting in production); the
ingest Lambda deploys and one curl becomes one object; DuckDB runs the scan
pack **over S3** and the S12 reconciliation identity closes with its terms
written out; then the compute dies the same sitting — and for the first
time in this course, **something is deliberately left standing.**
**Requirements:** S12 (+S2's plan-vs-real check; T5, T6 live) — task 1.8.
**Ramp you'll use:** Q's deploy-day discipline (transcripts land as they
happen; teardown is an acceptance criterion, not a chore), T's template
read (you know every resource about to exist), and the instant-stamping
habit T's scan pack drilled.

> **Honesty box (requirements.md):** this spec was authored with no AWS
> credentials, so nothing cloud-touching on this page could be validated in
> advance. Commands are real and desk-checked against the validated
> templates and binaries; cloud outputs are marked **(expected shape)** and
> your real ones replace them in `evidence.md`. Local commands (the
> dry-run, git, DuckDB against the local lake) keep the usual verbatim
> treatment — those *were* run.

## Where you are

Q's deploy day proved a handler; today's proves an *architecture*. The
laws you'll watch execute — `uploaded 0` on the second run, one curl one
object, the identity closing over `s3://` — are the same S3/S4/S6
statements that went green against the fake at 1,024 generated lakes in R
and T. Nothing on this page should surprise you; that's the point, and it
was the point of putting the seam where the un-testable thing starts. What
IS new is the ending: every deploy day until now finished at zero — stack
destroyed, log group deleted, nothing left but evidence. Today finishes at
*two buckets*: the lake stops being a directory in a git repo and becomes
infrastructure with a monthly bill (cents — approved at the gate) and no
teardown step. You own the credentials and every decision that costs
money; Claude co-drives, reads outputs with you, and drafts evidence as
you go.

## The build, move by move

All commands from the repo root unless noted. One commit at the end:
evidence. Cost up front (AI proposes, human disposes): a dozen-odd S3 PUTs
and LISTs, a handful of Lambda invocations, a few MB of CloudWatch —
fractions of a cent today; plus the one *recurring* line this course has
ever created: two SSE-S3 buckets holding ~150 KB of telemetry, **cents per
month at the outside** (the requirements' plain-words bound; the honest
arithmetic is fractions of a cent). The compute still tears down before
you stand up.

1. **Pre-flight (all boxes before any deploy).**

   ```console
   aws sts get-caller-identity        # the RIGHT account — your personal lab one
   aws configure get region           # us-east-1
   cd infra && npx cdk synth          # still exit 0 (re-gate any review edits from T)
   date -u +%FT%TZ                    # → evidence.md: deploy-day-two start timestamp
   ```

   Q's rules carry: `evidence.md` open in a buffer, transcripts land as
   they happen. One new rule for today, from T's scan pack: **every
   count you record gets its instant** — the lake grows while you work,
   and the identity in move 6 is built out of timestamped terms. And say
   the day's asymmetry out loud before you start: *the stateless stack
   dies before this sitting ends; the stateful stack must survive me.*
   Both halves get verified, not assumed.

2. **The lake is born (S12 begins) — then try to kill it.**

   ```console
   cd infra && npx cdk deploy goldeneye-stateful
   ```

   *(expected shape)* — one confirmation for the bucket policies (the
   enforceSSL denies you read in T; approve them as the things you
   reviewed), a short CloudFormation run, `✅ goldeneye-stateful`. Then
   verify the armor **against the live control plane**, not the
   template:

   ```console
   aws cloudformation describe-stacks --stack-name goldeneye-stateful \
     --query "Stacks[0].EnableTerminationProtection"
   # (expected shape) → true

   aws s3api get-bucket-encryption --bucket goldeneye-lake \
     --query "ServerSideEncryptionConfiguration.Rules[0].ApplyServerSideEncryptionByDefault"
   # (expected shape) → { "SSEAlgorithm": "AES256" }     ← the etag=md5 premise, live

   aws s3api get-public-access-block --bucket goldeneye-lake
   # (expected shape) → all four true
   ```

   Now the verification the course insists you *perform*, not read
   about — try to destroy the thing you just made:

   ```console
   npx cdk destroy goldeneye-stateful
   # (expected shape) → the CloudFormation refusal, verbatim into evidence:
   #   ValidationError: Stack [goldeneye-stateful] cannot be deleted while
   #   TerminationProtection is enabled
   ```

   Read what just happened as the AWS pro: the *control plane* refused
   you — the stack's first safety held against its own owner with full
   admin credentials, which is exactly the failure mode it exists for
   (the fat-fingered `destroy` in a tired terminal). The second safety
   (`RemovalPolicy.RETAIN` on each bucket) sits behind it, undemonstrable
   without actually flipping protection off — name it in evidence as
   verified-by-template (T's jq read) rather than by execution. One
   deploy-time fight to know about before it finds you: `goldeneye-lake`
   is a **globally unique** name and a hardcoded cross-spec contract —
   if creation fails with `BucketAlreadyExists`, someone on Earth owns
   the name, and the fix is a change-protocol event (the name appears in
   requirements, design, code, and queries), not a quiet rename in the
   stack file. Timestamp the birth in evidence.

3. **The backlog sync — plan, run, run again (S12's heart).** First the
   plan, and this command is local, credential-free, and was validated
   verbatim during authoring (your numbers WILL differ — the lake has
   grown since; that's the through-line, not a discrepancy):

   ```console
   cargo run -p lake-sync --release -- --bucket goldeneye-lake --dry-run datalake/raw-local
   ```

   Reference transcript (authoring instant, 2026-07-10):

   ```
   upload raw/dt=2026-07-05/events.jsonl (73771 bytes)
   upload raw/dt=2026-07-06/events.jsonl (1000 bytes)
   upload raw/dt=2026-07-08/events.jsonl (1001 bytes)
   upload raw/dt=2026-07-09/events.jsonl (33842 bytes)
   upload raw/dt=2026-07-10/events.jsonl (28412 bytes)
   upload raw/traces/dt=2026-07-05/02-collections-lens-28630.jsonl (14483 bytes)
   plan: upload 6, skip 0
   ```

   Copy YOUR plan into evidence with its instant, then the real thing —
   **back-to-back with the re-run**, because the identity depends on the
   lake not changing between them (a hook can fire mid-sitting):

   ```console
   cargo run -p lake-sync --release -- --bucket goldeneye-lake datalake/raw-local
   # (expected shape) → uploaded N, skipped 0        ← N matches your dry-run plan:
   #                                                    against an empty bucket, the
   #                                                    first-sync upper bound is exact
   cargo run -p lake-sync --release -- --bucket goldeneye-lake datalake/raw-local
   # (expected shape) → uploaded 0, skipped N        ← S4, in production
   ```

   That second line is the whole spec in ten characters. Savor it,
   then interrogate it like you would a customer's claim: what
   *specifically* proved idempotence — a flag? A local manifest? (No:
   the tool LISTed the bucket, compared your files by size-then-etag
   against what's really there, and found nothing to do. Delete the
   local tool's state — there is none — and it would still answer
   `uploaded 0`. The store is the source of truth; that was R's design
   decision executing.) If the re-run instead says `uploaded 1` naming
   today's partition — **that's not a bug, that's the design working**:
   a hook appended telemetry between your runs, today's `events.jsonl`
   changed, and the compare caught it. Note it with its instant and move
   on. Then look at the lake from the other side:

   ```console
   aws s3 ls s3://goldeneye-lake/raw/ --recursive
   # (expected shape) → your N objects, keys mirroring the local layout
   #                    VERBATIM — dt=*/, and traces/dt=*/ (S1, live)
   ```

   The quarantine note for evidence: if your lake has grown a
   `dt=bad-ts/` partition by now, confirm its object is there too — S1
   made relay's quarantine legal lake content, and the sync agreed.

4. **The ingest goes live — one curl, one object (S6 in production).**

   ```console
   npx cdk deploy goldeneye-stateless
   URL=$(aws cloudformation describe-stacks --stack-name goldeneye-stateless \
         --query "Stacks[0].Outputs[?OutputKey=='FunctionUrl'].OutputValue" --output text)
   FN=$(aws cloudformation describe-stacks --stack-name goldeneye-stateless \
        --query "Stacks[0].Outputs[?OutputKey=='FunctionName'].OutputValue" --output text)
   # capture FN NOW — move 7's orphan hunt needs it after the stack is gone
   ```

   Q's curl ritual, with a new observable — use a real envelope, and
   *predict its key first*: the object's day comes from the event's
   **ts**, not from today (the door hands the sink glake's day bucket).
   Q's `/tmp/one-event.json` was a `2026-07-09` event; its object will
   land **next to** the synced `events.jsonl` for that day, not in a
   today-partition:

   ```console
   head -1 datalake/raw-local/dt=2026-07-09/events.jsonl > /tmp/one-event.json
   jq -r '.event_id, (.ts[:10])' /tmp/one-event.json     # ← write the predicted key:
                                                         #   raw/dt=<day>/evt-<event_id>.json

   curl -si -X POST "${URL}events" -H 'content-type: application/json' \
        -d @/tmp/one-event.json | head -1
   # (expected shape) → HTTP/1.1 202 Accepted

   aws s3 ls "s3://goldeneye-lake/raw/dt=2026-07-09/"
   # (expected shape) → events.jsonl              ← the synced file (move 3)
   #                    evt-<event_id>.json       ← your curl, exactly at the predicted key
   ```

   Three probes while you're here, each one a requirement collecting:

   - **The retry story (T5):** fire the SAME curl again → 202, and the
     listing still shows **one** `evt-<event_id>.json` (its timestamp
     updated — the put replaced it; the key is the identity). What 006's
     stdout sink could never promise, key derivation gives for free.
   - **The reject story (S6):** `curl -s -X POST "${URL}events" -H
     'content-type: application/json' -d '{"not":"an envelope"}'` → the
     familiar 400, `{"error":"missing key: event_id"}` — and nothing new
     in the bucket (`aws s3 ls` count unchanged; rejected bodies put
     nothing). Keep the header on every curl, as in Q — the base64
     wrinkle is still armed without it.
   - **The content story (A4's rule, third venue):** pull the object
     and compare it to what you sent —

     ```console
     aws s3 cp "s3://goldeneye-lake/raw/dt=2026-07-09/evt-<event_id>.json" /tmp/landed.json
     diff <(jq -cS . /tmp/landed.json) <(jq -cS . /tmp/one-event.json)
     # (expected shape) → empty: one compact line, JSON-equal to the body
     ```

   Count your curls as you go — the accepted ones are a term in move
   6's identity. And spend one professional glance on what you did NOT
   have to configure: the function found the bucket via `LAKE_BUCKET`
   (the literal you reviewed), authenticated via its execution role
   (the one-statement policy — `PutObject` on `raw/*` only), and if you
   try to make it misbehave, the IAM layer is the backstop: it *cannot*
   list, read, or write outside `raw/` no matter what a future code bug
   wants. That's S1 enforced twice — in code and in IAM — which is what
   the hand-written statement bought.

5. **DuckDB over S3 (T6 — the same questions, the durable answers).**
   The `s3/` variants differ from the `local/` ones you ran in T only in
   preamble and paths (you diffed them; that was the proof). The
   preamble needs the `httpfs` extension and a `credential_chain`
   secret — the same chain your CLI just used:

   ```console
   python3 -c "import duckdb; print(duckdb.sql(open('datalake/queries/duckdb/s3/01-sanity-event-inventory.sql').read()))"
   # (expected shape) → your event-type inventory — the same table T's local
   #                    run gave you, now answered by the bucket
   #                    (+1 to the type of your curl-ingested event: the pinned
   #                    glob raw/dt=*/*.json* reads events.jsonl AND evt-*.json)
   ```

   No server, no cluster, no Athena — an extension and a credential
   chain, aimed at a bucket. That's the whole T6 lesson, and it's why
   the scan pack could be written and validated locally first: SQL that
   only differs in paths is SQL you can trust in both places.

6. **The S12 reconciliation identity — terms written out.** T taught the
   local identity; deploy day adds one term. The S3 side counts what the
   glob matches — the synced `events.jsonl` files **plus every
   `evt-*.json` your curls minted** — so:

   > **S3 process events (at T₂) = local process events (at the sync
   > instant T₁) + accepted curl-ingests since (T₁→T₂)**
   > and `trace_rows(s3) = trace_rows(local at T₁)` — nothing curls into
   > `traces/`.

   Run query 08 in both variants back-to-back and fill the table —
   every term with its instant (this template goes in evidence.md
   verbatim; NOTES §6 asked for exactly this):

   ```markdown
   | Term | Value | Instant |
   |---|---|---|
   | local process_events (s08 local, at sync) | | T₁ = |
   | local trace_rows / trace_newlines         | | T₁ |
   | glake stats total (local, back-to-back)   | | T₁ |
   | accepted curl-ingests (your count)        | | T₁→T₂ |
   | s3 process_events (s08 s3)                | | T₂ = |
   | s3 trace_rows / trace_newlines            | | T₂ |
   | **identity: local + curls = s3 side**     | ✓/✗ | |
   ```

   For calibration, the two *local* instants this course has already
   recorded: 350 + 64 = 414 (authoring, 2026-07-10T23:24Z) and
   354 + 64 = 418 (materials verification, an hour later). Yours is a
   third instant; the *identity* is what must hold, never the numbers.
   If it misses: by exactly your curl count → you forgot the new term
   (the glob reads `evt-*.json` too); by a handful → a hook fired after
   the sync — the local side moved and S3 is honestly behind; re-sync
   and re-run, or state both instants and show the difference equals
   the local growth. Either way, write the *reconciliation*, not just
   the verdict — that habit is the R10 lesson at full scale, and it's
   what makes this evidence auditable in six months.

7. **Teardown — and the ending that's different (S12 closes).**

   ```console
   date -u +%FT%TZ                          # evidence: destroy timestamp
   cd infra && npx cdk destroy goldeneye-stateless
   # (expected shape) → Are you sure…? y → goldeneye-stateless: destroyed

   aws cloudformation describe-stacks --stack-name goldeneye-stateless
   # (expected shape) → Stack … does not exist

   # Q's orphan, collected again (you captured $FN in move 4):
   aws logs describe-log-groups --log-group-name-prefix "/aws/lambda/$FN" \
     --query "logGroups[].logGroupName"
   # (expected shape) → it's STILL THERE — service-created, invisible to
   #                    cdk destroy. Delete it; teardown you didn't verify
   #                    isn't teardown (Q's lesson, now a reflex):
   aws logs delete-log-group --log-group-name "/aws/lambda/$FN"
   ```

   And now the verification Q never had — **that the right thing
   survived**:

   ```console
   aws cloudformation describe-stacks --stack-name goldeneye-stateful \
     --query "Stacks[0].StackStatus"
   # (expected shape) → "CREATE_COMPLETE"      ← untouched by the stateless destroy
   aws s3 ls s3://goldeneye-lake/raw/ --recursive | wc -l
   # (expected shape) → your N + curl objects — ALL still there
   ```

   The compute is gone; the lake is not; and no CloudFormation edge
   connected them for the destroy to trip over (T's no-cross-stack
   decision, paying out). The Function URL is dead, so the NONE-posture
   exposure you re-examined in T is closed with it — note that in
   evidence next to your T finding. Finish the evidence file: deploy
   and destroy timestamps; the plan/run/re-run transcripts; the
   one-curl-one-object listing with the predicted key; the identity
   table with instants; the cost note — PUT requests (N sync + curls ≈
   a cent's small change), Lambda invocations (Q-scale, ≈ $0.00),
   CloudWatch MBs, and the recurring line: ~150 KB across two SSE-S3
   buckets, fractions of a cent per month, bounded "cents" in the
   approved requirements. Then:

   ```console
   git add specs/007-lake-to-s3/evidence.md
   git commit -m "007: sitting U — deploy day two: lake born and armored, backlog synced, uploaded 0 proven, compute down"
   ```

   (Close-out — property-auditor, SKILLS/MEMORY updates, the
   `spec-close/007-lake-to-s3` marker, datalake/README's Wave-2 flip —
   is task 2.1, Claude's machine work, next session.)

## Fights to expect

No compiler today — platform fights, most of which you've debugged for
other people. Ledger the ones that still bite (`learning.*`).

- **The double-slash 404** — Q's classic, still armed: the URL output
  ends in `/`, so it's `${URL}events`, never `$URL/events`.
- **`BucketAlreadyExists` on the stateful deploy** — the global
  namespace said no. Move 2's rule: the name is a cross-spec contract;
  changing it is a change-protocol event, not an improvisation.
- **The sync exits 2 with an SDK error before any upload** — that's the
  LIST failing, and it runs as *your* credentials, not the Lambda's
  role: the operator needs `s3:ListBucket` + `s3:PutObject` on the lake
  (an admin lab profile has both; a scoped CI-style profile may not).
  The exit-code boundary is telling you which phase died: 2 = couldn't
  even try (plan/LIST), 1 = tried and some puts failed, partial
  progress named.
- **`uploaded 1` on the re-run** — not a bug: the hooks appended to
  today's partition between your runs (move 3). The compare caught a
  real change; record the instant and let it.
- **DuckDB-over-S3 refuses** — triage in order: `INSTALL httpfs;` needs
  network once per DuckDB install; the secret's `credential_chain` pulls
  whatever `aws sts get-caller-identity` sees — if that's empty, DuckDB
  is too; a 403 with valid credentials is usually region (the bucket is
  us-east-1; a mis-set `AWS_REGION` redirects the endpoint); and "no
  files found" on a correct bucket means the glob — check you kept
  `raw/dt=*/*.json*` verbatim (the `*` before `.json*` must survive
  shell-quoting into the SQL file, which is why the preamble lives *in*
  the file).
- **The identity misses** — move 6's triage: exactly-your-curl-count
  means the forgotten term; a-handful means instants; write the
  reconciliation either way.

## Checkpoint

The sitting counts as done only when all of these hold:

```
aws cloudformation describe-stacks --stack-name goldeneye-stateful \
  --query "Stacks[0].[StackStatus,EnableTerminationProtection]"
                                     # → CREATE_COMPLETE, true — alive and armored
aws cloudformation describe-stacks --stack-name goldeneye-stateless
                                     # → does not exist (compute torn down)
aws s3 ls s3://goldeneye-lake/raw/ --recursive | wc -l
                                     # → N + curls — the lake holds everything
git log --oneline -1                 # → 007: sitting U — deploy day two: …
```

- `specs/007-lake-to-s3/evidence.md` holds, with real values and
  instants: both deploy timestamps + the destroy timestamp; the
  termination-protection refusal transcript; the plan → `uploaded N` →
  `uploaded 0` triple; the one-curl-one-object listing (predicted key
  and landed key, equal); the retry and reject probes; the identity
  table with every term timestamped; the cost note including the
  recurring cents line; and the survived-teardown verification.
- You can answer aloud: what did `uploaded 0` actually check, and where
  does that state live? (Nowhere local — that's the answer.) Why did
  the curl's object land in a July partition regardless of today's
  date? Which two mechanisms protect the lake from deletion, and which
  one did you *watch* work? Why does the S3-side event count exceed the
  local count, by exactly what, and what would each kind of mismatch
  mean? What's the one AWS resource this course now leaves running,
  what does it cost, and where was that cost approved? And the arc
  question, for the spec close: R proved these laws against a fake at
  1,024 lakes in four seconds — what, specifically, did today add to
  that proof? (The translation layer and the account — and *nothing
  else*, which is why today held no surprises. Say it like you'd say it
  in a design review.)

## Hints (one at a time)

<details><summary>Hint 1 — capturing the refusal and the survivals cleanly</summary>

The termination-protection refusal comes back as a CLI error, so grab
it with stderr redirection for evidence:
`npx cdk destroy goldeneye-stateful 2>&1 | tail -5` — you want the
`ValidationError … TerminationProtection is enabled` line verbatim
(and note *when* it arrived: after you answered `y` to the
confirmation prompt — CloudFormation refused even with your consent
given, which is the point: human consent was not the last line of
defense). For the survived-teardown pair, one compound check reads
well in evidence:

```console
aws s3api head-bucket --bucket goldeneye-lake && echo "lake: alive"
aws s3api head-bucket --bucket goldeneye-discovery && echo "discovery: alive"
```

`head-bucket` is the cheapest liveness probe S3 has (a bare HEAD — no
listing, no bytes), and it exits non-zero on missing/forbidden, so the
`&&` echo only prints for a bucket that's really yours and really
there.

</details>

<details><summary>Hint 2 — the ingested object won't show up where you predicted</summary>

Work the key backwards. (1) Was the curl actually a 202? A 400 puts
nothing (check the body — the door's error names the first problem).
(2) Re-derive the prediction: day = the event's `ts[:10]` (NOT today —
the door hands glake's bucket to the sink), id = the `event_id`
sanitized to `[A-Za-z0-9._-]`; if your test event's id carries odd
characters, the landed key spells them as `_` (T's sanitizer test has
the exact rule). (3) List one level up — `aws s3 ls
s3://goldeneye-lake/raw/ --recursive | grep evt-` — if the object is
under an unexpected `dt=`, read the event's ts again; a `bad-ts` event
can't be there at all (the strict door 400s it — quarantine is
lake-sync's business, not ingest's). (4) Still nothing, but the curl
202'd? That would be a durability lie, which the 500 branch exists to
prevent — check the function's CloudWatch stream for a
`put failed after accept` error line: an AccessDenied there means the
key escaped `raw/*` (the IAM backstop working) and is a genuine
finding to bring to session.

</details>

## If truly stuck

There is no reference deploy to compare against — the honesty box is
the deal — but the paper trail covers everything up to the account
boundary:

- `specs/007-lake-to-s3/_reference/NOTES.md` §1, §3, §5 — the exact
  artifacts, templates, and local query outputs that are deploying; if
  the cloud behaves surprisingly, diff reality against these first.
- `infra/README.md` — deploy/destroy commands and the two stacks'
  different lives ("goldeneye-stateful is never in a destroy command").
- `datalake/queries/duckdb/README.md` — the httpfs preamble, the
  credential chain, the pinned glob's rationale, and the identity's
  deploy-day term.
- `specs/007-lake-to-s3/requirements.md` — S12's exact wording, which
  is what evidence.md is answering to (buckets retained, compute torn
  down, identity with terms, costs and timestamps).

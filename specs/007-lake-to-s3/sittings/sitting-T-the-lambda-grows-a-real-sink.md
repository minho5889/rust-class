# Sitting T — the Lambda grows a real sink

**Builds:** the exile ends. `crates/hello-lambda` evolves at exactly the
seam sitting O built for this day: the accepted line goes to
`store.put(raw/dt=<day>/evt-<event_id>.json, line)` instead of `println!`
(S6) — the `Emit` trait dies a structural death, the handler core goes
generic, the client is built once in `main` and parked in a `OnceLock`, and
a put that fails answers **500, never a durability-lying 202**. H4 re-runs
green: the door did not move. Then two ceremonies this course has been
saving: your first **change-protocol edit of a gated doc** (S6b — 006's H1
and H7 get their supersession entries, supervised), and the **stateful
stack review** — Claude wrote the lake's CDK home; you tear into it as the
AWS professional before anything deploys. The scan pack closes the sitting:
eight standing queries, run by you, reconciled against your own walker.
**Requirements:** S6, S6b, S10, S11 (+S8 closes) — tasks 1.6–1.7.
**Ramp you'll use:** sitting O's emit-seam design ("this is also 007's swap
point" — that sentence, coming due), P's stack-review ritual (you
interrogate, buckets: understood / challenged / flagged), J's F13
monomorphization, and 003 R10's reconcile-don't-compare.

## Where you are

S ended with the bill drafted and one row deliberately empty. The seam has
two implementations, fifteen green tests, and a CLI — but the *Lambda*
still prints to stdout, because that's all 006 was allowed to do (H7 banned
the SDK on purpose, to keep the baseline clean for the very measurement you
just made). Today the ban lifts — **through the front door**. That's what
S6b is really about: 006's requirements are approved, gated documents, and
two of their lines are about to become false. The amateur move is to
quietly make the code disagree with the spec; the pipeline's move is to
amend the upstream doc with a changelog entry and re-gate the amendment.
You'll do that today, in 006's own file, with your own hands — the one
place in this course a learner edits a gated doc, and the protocol is the
lesson.

The other half of today is review, not writing: the stateful stack is
Claude's code, ~90 lines of TypeScript that will hold your telemetry
*permanently*. You've reviewed a hundred stacks like it for customers.
This one you review as the owner — and there's a finding in it the course
wants *you* to raise, not be told about.

## The build, move by move

All commands from the repo root. Five commits: sink swap, tests green +
the bill closed, the 006 amendment, infra review notes, scan pack outputs
— tasks 1.6 and 1.7.

1. **Collect on O's promise (task 1.6 begins).** Open your
   `crates/hello-lambda/src/handler.rs` and re-read the emit seam you
   built in sitting O — the `Emit` trait, `StdoutEmit`, and the guide's
   sentence: *"This is also 007's swap point — an S3-backed sink replaces
   `StdoutEmit` without touching the door or the routing."* Before
   changing anything, answer aloud: what exactly did the seam promise?
   (The door *returns* the line; the sink decides what a line becomes;
   202 means "validated and handed to the sink.") What does 202 have to
   mean once the sink is a *bucket*? (Handed to the sink now means
   **landed in the lake** — and that has a failure mode stdout never had.
   Hold it; move 3 collects.) And the design question the swap forces:
   `Emit` was `&dyn` — can the new seam be? You already know from R:
   `ObjectStore` is RPITIT, **not dyn-compatible** (E0038), and its `put`
   is async where `emit` was sync. So the handler core goes **generic** —
   `handle_with<S: ObjectStore>(req, store: &S)` — monomorphized over
   `S3Store` in production and `FakeStore` in tests: F13's pattern, third
   appearance, and this time it isn't a style choice. The `Emit` trait is
   **deleted, not deprecated** — carrying a dead seam "for compatibility"
   would be exactly the kind of kindness 004 taught you to refuse.

2. **The swap itself.** Three edits inside `handler.rs`, none of which
   touch the door's verdicts:

   - **The door hands over more than a line.** 006's `door` returned
     `Result<String, String>` — but look at what it already *had* in
     hand and dropped on the floor: glake's classification carries the
     `day`, and the re-serialize parse holds the `event_id`. Give the Ok
     side a real name: `Accepted { line, day, event_id }`. (The bad-ts
     sentinel never reaches the Ok arm — the strict door refused it —
     so `day` is always a real `YYYY-MM-DD`.) One honesty note at the
     `event_id` extraction: presence-checking never promised a *string*.
     A numeric or structured id gets its compact JSON rendering — an
     honest spelling that still derives a stable key. Document the
     decision at the extraction site.
   - **The key (S1's ingest half):**
     `raw/dt=<day>/evt-<event_id>.json` — one event, one object,
     **self-idempotent under retry because the id IS the address**.
     Contrast it with what you earned in R/S: lake-sync had to *work*
     for idempotence (list + size/etag compare); ingest gets it free
     from key derivation. Same law, two prices. But `ingest_key` must
     defend one thing: the scan pack's glob is pinned as
     `raw/dt=*/*.json*` (S11), and an `event_id` containing `/` would
     nest the object a level deeper and **silently escape every standing
     query**. Sanitize to `[A-Za-z0-9._-]`, everything else `_` — for
     hook-minted ids (digits and dashes) it's a no-op; for hostile or
     buggy ones it keeps the object where the queries look. Write the
     unit test now: the real envelope's id passes through untouched;
     `"a/b c\"d"` becomes `a_b_c_d`; a numeric id keys as its rendering.
   - **The put, and the new failure.** The accepted arm becomes:
     derive the key, `store.put(key, line.into_bytes()).await`, and
     *match the result*. `Ok` → count accepted, 202, empty body —
     unchanged. `Err` → the branch 006 could not have: the event was
     valid but did not land. Answering 202 would claim durability you
     don't have — relay's writer-unavailable lesson, now with money on
     the table — so it's the crate's one **500**,
     `{"error":"lake unavailable"}`, diagnostics to stderr via
     `tracing::error!`, never stdout. And say what this does to the
     healthz arithmetic *before* the test forces you to: a put-failed
     request is neither `accepted` (not in the lake) nor `rejected`
     (the door said yes) — so **`received = accepted + rejected` holds
     on failure-free runs only**. That invariant quietly weakened; pin
     the new truth with a dedicated test and say it in the healthz docs
     (H3a's counters are yours to keep honest).

   The manifest: add `lake-store = { path = "../lake-store" }` and
   `aws-config = "1"` to dependencies; `lake-store` with
   `features = ["fake"]` to dev-dependencies. Notice what you did *not*
   add: `aws-sdk-s3`. The SDK arrives through the seam's re-export and
   stays named in exactly one crate (S7, extended to the dependency
   graph). **Commit point:**

   ```console
   cargo fmt && cargo clippy -p hello-lambda --all-targets -- -D warnings
   git add crates/hello-lambda Cargo.lock
   git commit -m "007: sitting T — sink swap at the 006 emit seam, put-failure is a 500"
   ```

   (Tests are red-to-missing right now — the old H1 Vec-sink tests
   don't compile against the new seam. That's the next move, not a
   broken commit: clippy and the lib build are clean.)

3. **`main.rs` — everything expensive happens once (T3).** The shape to
   build toward, and the cold-start argument for each line: subscriber →
   stderr first (stdout lost its protocol job — accepted events go to S3
   now — but the discipline and the CloudWatch layout stay; reclaiming
   stdout for noise would be the wrong lesson to unlearn); eager
   `state::instance()` touch (H3a/H10, unchanged); then the new part —
   `LAKE_BUCKET` from the environment (the CDK stack sets it; a missing
   one is a *deploy* bug, so fail init loudly rather than limp into
   serving 500s with a made-up name), `aws_config::load_defaults(…).await`
   **once**, build **one** `S3Store`. Config loading resolves the
   credential chain and builds a connection-pooled client — an async,
   potentially slow dance. Per-request would tax every event; lazy would
   hide it in someone's first-request latency; in `main` it lands in the
   init phase, once per instance — the same reason relay bound its
   listener before announcing "listening on".

   Now the fight. The handler takes two arguments, so `service_fn` needs
   a closure, and the natural spelling walks into a wall (verified,
   verbatim):

   ```
   error: lifetime may not live long enough
     --> src/main.rs:…
      |
      |     run(service_fn(move |req| handler::handle_with(req, &store))).await
      |                    ---------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ returning this value requires that `'1` must outlive `'2`
      |                    |        |
      |                    |        return type of closure `impl Future<Output = …>` contains a lifetime `'2`
      |                    lifetime `'1` represents this closure's body
      |
      = note: closure implements `Fn`, so references to captured variables can't escape the closure
   ```

   Read the note line — it's the whole story: the closure *owns* the
   store (you moved it in), each call returns a future that *borrows*
   `&store` from the closure's capture, and an `Fn` may be called again
   while an earlier future still lives — so a reference to a captured
   variable may not escape into the return value. The future needs a
   store reference that outlives every call: a **`&'static S3Store`**.
   And you already own the tool that mints one — the same `OnceLock`
   pattern `state.rs` has used since O, now doing the same job for a
   value that must be *constructed asynchronously first*:

   ```rust
   static STORE: OnceLock<S3Store> = OnceLock::new();
   // …in main, after the config dance:
   let store: &'static S3Store =
       STORE.get_or_init(move || S3Store::new(S3Client::new(&config), bucket));
   run(service_fn(move |req| handler::handle_with(req, store))).await
   ```

   No `Box::leak` trick, no global constructor, no unsafe — a static
   cell, written once in `main`, read by every request the instance ever
   serves. (Monomorphization footnote: this `handle_with` call pins
   `S = S3Store`; the tests' calls pin `S = FakeStore`. Two copies, no
   vtable, both checked against the same trait — F13, again.)

4. **Tests: the S6 suite, the H4 surgery, and the bill's last row (task
   1.6 closes).** `tests/handler.rs` keeps 006's skeleton (the `SERIAL`
   tokio-Mutex, delta assertions, the real-envelope fixture — three
   specs provably talking about the same traffic) and re-aims the
   observations: where H1's tests read a Vec sink, S6's read the fake's
   ledger. The suite to build — each row names its requirement:

   - **S6:** valid body → 202, empty body, **exactly one put, at the S1
     key**, holding **one compact line** JSON-equal to the fixture.
   - **S6, multiline:** the pretty-printed twin spans lines as a body
     but lands as one compact line — *in the same object* (the key is
     the identity).
   - **S6, the retry story (T5):** the same event twice = two puts, ONE
     key, one object — the handler doesn't dedup; **the key does**.
   - **H2, unchanged:** the three 400 clauses in the fixed order, byte-
     for-byte relay's error strings — and **zero puts** for all of them.
   - **S6's failure edge:** `fail_on` the fixture's key → 500,
     `{"error":"lake unavailable"}`, nothing stored, and the counter
     deltas prove the weakened invariant: received +1, accepted +0,
     rejected +0.
   - **H3/H3b, unchanged:** healthz shape and deltas (plus the v1
     addendum: two accepts whose bodies differ only in clock-time are
     two puts to the *same* key — same day, same id — so the store holds
     ONE object; identity ignores time-of-day on purpose); everything
     else 404s, counters still, nothing put.

   Then the property. Open `tests/prop_door.rs` and perform surgery, not
   a rewrite: the generator, the relay oracle, the live-receiver rule,
   the 64 KiB domain bound — all untouched. `hello_verdict` swaps its Vec
   sink for a fresh `FakeStore` per body and returns the put-log; the
   rider assertions become S6-flavored — **one put iff 202, zero
   otherwise, and every put key is S1-shaped** (`raw/dt=` prefix,
   `.json` suffix, exactly two `/`s — inside the pinned glob's reach).
   Run it and read the sentence the transcript writes (verified):

   ```
   running 7 tests
   test h2_invalid_bodies_400_in_fixed_order_and_put_nothing ... ok
   test s6_multiline_body_stored_as_one_compact_line_same_key ... ok
   test h3_healthz_shape_and_counter_deltas ... ok
   test h3_everything_else_is_404 ... ok
   test s6_put_failure_answers_500_not_a_lying_202 ... ok
   test s6_retried_event_overwrites_itself ... ok
   test s6_valid_event_202_one_put_at_the_s1_key ... ok

   running 1 test
   test prop_h4_door_equivalence ... ok
   test result: ok. 1 passed; … finished in 0.55s
   ```

   **H4 green after the evolution is the sitting's quiet headline: you
   replaced the sink with a bucket and relay couldn't tell.** 512 cases,
   a few thousand verdict pairs, half a second — the door did not move,
   and now that's a proof, not a hope (S6's re-run clause, satisfied).

   Now collect on your S prediction — the bill's empty row:

   ```console
   cargo lambda build --release --arm64 -p hello-lambda
   ls -l target/lambda/hello-lambda/bootstrap
   ```

   Reference numbers (validated; your lock may drift a patch):
   **11,431,848 bytes** against P's 1,843,256 — **+9,588,592 bytes,
   +9.14 MiB, ×6.2**. The constitution's "~10 MB+" warning, measured and
   survived. Run the bloat read for the evidence table
   (`cargo bloat --release --crates -n 10 -p hello-lambda`) — reference
   top rows: std 1.9 MiB, **aws_lc_sys 1.3 MiB, h2 970.3 KiB,
   aws_smithy_runtime 578.8 KiB, rustls 529.2 KiB** … and `aws_sdk_s3`
   itself at **327.5 KiB**. Same punchline as S, now on the artifact
   that cold-starts: **you paid for the transport, not the client** —
   and Q's cold-start table is the reason to care: that 9 MiB is what
   Linux now execs per cold start. (A worthwhile follow-up measurement
   for deploy day: does init move from Q's numbers? Write your
   prediction in the evidence draft.) Fill the row, update the delta,
   note `cargo tree -p hello-lambda -e normal | grep -c aws-sdk` now
   answers **4** (s3 + aws-config's sso/ssooidc/sts providers — expect
   the family, not the one you named). **Commit point:**

   ```console
   git add crates/hello-lambda specs/007-lake-to-s3/evidence.md
   git commit -m "007: sitting T — S6 suite green, H4 re-proven, the bill closed at x6.2"
   ```

5. **The change-protocol moment (S6b) — you edit a gated doc,
   supervised.** Two lines of 006's approved requirements are now false
   about the evolved crate: **H1** promises the accepted event lands as
   "exactly one compact JSON line on stdout" (the sink you just retired),
   and **H7** declares "No `aws-sdk-*`" (the tree now says 4). The
   protocol (CLAUDE.md, "Change protocol"): halt the contradicting work —
   done, it's committed but not yet gated; **amend upstream with a
   changelog entry**; re-gate *only the amendment*. S6b pre-authorized
   exactly this moment, so the amendment is an execution, not a debate.

   Open `specs/006-hello-lambda/requirements.md`. Do **not** rewrite the
   H1/H7 body text — the doc as approved is the historical record, and
   its sittings (O–Q) were run against it as written. The amendment
   lives in the changelog table at the bottom. Append these two rows
   (today's date):

   ```markdown
   | 2026-…-… | **H1 superseded at the emit seam** (007 S6/S6b): the stdout sink retires — accepted events now land as S3 objects (`raw/dt=<day>/evt-<event_id>.json`) via the ObjectStore seam. H1's door behavior (202/empty body/one compact line) is unchanged and still pinned by H4, re-run green post-evolution. | 007 sitting T, per S6b's change protocol | with 007's close |
   | 2026-…-… | **H7's "no aws-sdk-*" superseded for the evolved crate** (007 S6b): lake-store + aws-config arrive through the seam (`cargo tree` count 0 → 4). The no-SDK baseline is preserved as the 006 reference artifact and the S8 bill's baseline row; H7's lint clauses (unwrap/expect denied, stderr rule) remain in force. | 007 sitting T, per S6b's change protocol | with 007's close |
   ```

   Read your own rows back against the protocol's checklist: does each
   name *what* is superseded, *what survives* (the door, the lints, the
   baseline artifact), *who authorized it* (S6b), and *where the re-gate
   happens* (007's combined close — these rows ride to the human gate
   with this spec, they don't self-approve)? That last cell is the
   teeth: you've amended, not approved. The 006 doc's `**Status:**` line
   stays `approved` — the amendment is what awaits the gate. **Commit
   point** (its own commit, always — an amendment buried in a code
   commit is an amendment nobody reviews):

   ```console
   git add specs/006-hello-lambda/requirements.md
   git commit -m "007: sitting T — 006 H1/H7 supersession entries (S6b change protocol)"
   ```

6. **The stateful stack — review it like it's going to hold your data
   forever, because it is (task 1.7).** Claude drove; you now read
   `infra/lib/stateful-stack.ts` end to end (~90 lines), plus the new
   stanza in `infra/bin/goldeneye.ts`, P's ritual: every line lands in a
   bucket — *understood, challenged, or flagged*. The interrogation
   list, pro to pro:

   - **Two safeties, two failure modes.** `terminationProtection: true`
     sits on the *stack* (at the `bin/` instantiation — find it): `cdk
     destroy` refuses until a human flips it off in a separate,
     deliberate step. `RemovalPolicy.RETAIN` sits on each *bucket*: even
     if the stack somehow dies, CloudFormation orphans the buckets
     instead of emptying the lake. Name a scenario each one catches that
     the other doesn't — if you can't, the redundancy is theater; if you
     can, it's engineering. (Sitting U tests the first one against the
     real control plane.)
   - **SSE-S3, not KMS — and it's a *correctness* decision.** You read
     the honest paragraph in S: single-part PUT under SSE-S3 is the
     premise that makes etag = md5, which is what makes `uploaded 0`
     provable. Flipping this bucket to SSE-KMS would silently turn
     idempotent syncs into upload-everything-every-run. An encryption
     setting that can break a *law* — say it in your review note,
     because whoever "hardens" this bucket in two years won't know.
   - **Versioning off, explicitly** (S10): the lake is append-shaped —
     per-file sync, per-event ingest, no deletes in Phase 1 — so
     versions would be noise with a bill. Challenge it anyway: what
     would versioning buy against the failure modes you actually have?
     (A fat-fingered local deletion doesn't propagate — sync never
     deletes. The honest answer is "nothing, at this workload.")
   - **No cross-stack reference — on purpose.** The stateless stack
     sets `LAKE_BUCKET=goldeneye-lake` as a *literal* and writes the
     ARN by hand, instead of importing the bucket construct. The
     hardcoded name IS the cross-spec contract (one of the project's two
     name exceptions), and zero CloudFormation coupling means compute
     teardown can never tangle with lake state. You've seen the
     alternative fail in tickets: export/import lock-ups where a dead
     dev stack holds a data stack hostage.
   - **The hand-written grant (S10's headline).** In
     `stateless-stack.ts`, the ingest permission is a `PolicyStatement`
     written by hand — Sid `GoldeneyeLakeRawPutOnly`, **one action**
     (`s3:PutObject`), **one prefix** (`arn:aws:s3:::goldeneye-lake/raw/*`).
     The idiomatic `lake.grantPut(handler)` was rejected, and the
     comment names why: grantPut bundles `s3:PutObjectLegalHold`,
     `PutObjectRetention`, `PutObjectTagging`, `PutObjectVersionTagging`
     and `s3:Abort*` — none used here, and their presence would falsify
     the requirement's "nothing wider" claim. Least privilege you can
     literally read in the template. Verify the claim in move 7's synth,
     not in the TypeScript.
   - **The acknowledgments.** The discovery synth (acknowledgments
     disabled) reported exactly three findings — read each written
     reason like the suppression ticket it is:
     `AwsSolutions-IAM5[Resource::arn:aws:s3:::goldeneye-lake/raw/*]`
     (the wildcard IS the least privilege: per-event keys are minted at
     request time, so no finite resource list can exist — one action,
     one prefix; hold or fold?), and `AwsSolutions-S1` twice (no access
     logs: a single-operator lab lake, BPA'd and SSL-enforced, where the
     log bucket would itself flag S1 and double the stateful footprint
     to audit cents of telemetry). And the tooling wart, third data
     point for 006's upstream-issue candidate: the granular IAM5 id
     embeds the ARN's `::`s, which `Validations.acknowledge()` rejects
     (qualifyId reserves the delimiter) — so that one acknowledgment
     travels via the public `ACKNOWLEDGED_RULES_METADATA_KEY` channel,
     exactly like 006's IAM4. Worse, the CLI's *suggested* ack strings
     are wrong in both directions (it suggests a `Pack::RuleId` form the
     matcher doesn't match). When a tool's own suggestion won't
     round-trip through its own API, real-rule-IDs-from-real-output is
     the only discipline that survives.

   **And now the finding you're supposed to catch.** Scroll to the
   AuthType=NONE justification block — the one *you* acknowledged in P —
   and re-read bound 2: *"blast radius: the function writes only to its
   own CloudWatch log group; it can reach no data store, no lake, no
   other resource (the role is logs-only, and 006 deliberately ships no
   aws-sdk)."* Every clause of that sentence was true when you signed
   it. Is any of it true now? The role you just verified carries
   `s3:PutObject` on the lake; the binary ships the SDK; a public,
   unauthenticated URL now fronts a function that **writes durable
   objects into the permanent bucket**. The bounds that remain: it can
   only *add* (no read/list/delete), only under `raw/`, only
   valid-envelope-shaped bodies, only for the sitting it's deployed
   (H11's teardown still applies to the compute), at cents of worst-case
   cost. Your call to make, in writing: does the NONE posture still
   hold under the new blast radius, or does it need AWS_IAM now? Either
   answer is defensible; an *unreviewed* posture is not. Log it as a
   review finding in your notes and evidence — the H12 comment's bound 2
   is stale and is a candidate for amendment at this spec's close
   (exactly the class of gap P said written justifications exist to
   surface — this time it surfaced on schedule, one spec later).

7. **Synth, and read the templates like a diff (task 1.7 continues).**

   ```console
   cd infra && npx tsc --noEmit        # exit 0
   npx cdk synth                        # BOTH stacks now; bundles your evolved crate
   jq '.pluginReports' cdk.out/validation-report.json    # [] — zero unacknowledged
   ```

   The assertion rounds below were run against the validated reference
   synth — your logical IDs will match, hashes may differ:

   ```console
   jq '.Resources | map_values(.Type)' cdk.out/goldeneye-stateful.template.json
   ```
   ```json
   {
     "LakeBucket9CD7BBD2": "AWS::S3::Bucket",
     "LakeBucketPolicy7D3A2454": "AWS::S3::BucketPolicy",
     "DiscoveryBucket0A653B1C": "AWS::S3::Bucket",
     "DiscoveryBucketPolicy1B7594F0": "AWS::S3::BucketPolicy",
     "CDKMetadata": "AWS::CDK::Metadata"
   }
   ```

   Five resources, and you can account for all of them (the two bucket
   policies are `enforceSSL`'s deny-on-`aws:SecureTransport=false` —
   free TLS enforcement that also pre-satisfies AwsSolutions-S10, so no
   suppression needed). Per bucket, verify with your own eyes (verified
   values shown): `DeletionPolicy` AND `UpdateReplacePolicy` both
   `"Retain"` — the second one matters and juniors miss it: a *rename or
   replacement* update would otherwise delete the old bucket;
   `SSEAlgorithm: "AES256"`; all four `PublicAccessBlockConfiguration`
   flags `true`; **no** `VersioningConfiguration` key at all (off is
   spelled as absence); the `project=goldeneye` tag. Stack-level:
   `"terminationProtection": true` lives in `cdk.out/manifest.json`
   (it's a CFN *stack* property, not a resource — know where to look).
   Then the stateless diff — what 007 changed in the stack you already
   reviewed in P:

   ```console
   jq '[.Resources[] | select(.Type=="AWS::IAM::Policy")][0].Properties.PolicyDocument.Statement' \
       cdk.out/goldeneye-stateless.template.json
   ```
   ```json
   [
     {
       "Action": "s3:PutObject",
       "Effect": "Allow",
       "Resource": "arn:aws:s3:::goldeneye-lake/raw/*",
       "Sid": "GoldeneyeLakeRawPutOnly"
     }
   ]
   ```

   Exactly one statement, one action, one prefix — the hand-written
   claim, surviving to CloudFormation (this is the artifact your grantPut
   conversation was about; a grantPut version would show five-plus
   actions here). And the function grew
   `"LAKE_BUCKET": "goldeneye-lake"` in its environment — the literal,
   not an import. `Architectures: ["arm64"]`, still explicit, still
   load-bearing. **Commit point:**

   ```console
   git add specs/007-lake-to-s3/evidence.md    # + infra/ if your review drove edits
   git commit -m "007: sitting T — stateful stack reviewed: RETAIN x2 verified, NONE posture re-examined"
   ```

8. **The scan pack — run your gradebook (task 1.7 closes, S11).**
   `datalake/queries/duckdb/` holds the eight standing queries in two
   variants; today you run all eight `local/` ones yourself against the
   real lake. The runner (duckdb CLI or the python module, whichever
   your machine has):

   ```console
   python3 -c "import duckdb; print(duckdb.sql(open('datalake/queries/duckdb/local/01-sanity-event-inventory.sql').read()))"
   ```

   Three rules to check as you go, each a telemetry-constitution clause:
   every file's header names the doc it feeds (`-- feeds:`); the events
   relation pins its columns (`read_json` with `columns={…}`, never
   `read_json_auto` — auto-inference *samples*, and `payload`'s inferred
   type flipped between JSON and MAP run-to-run as the reference lake
   grew; a standing query must not have weather-dependent schema); and
   the s3 twins differ **only** in preamble and paths — prove that one
   mechanically: `diff local/01-… s3/01-…` and read what's left (the
   httpfs INSTALL/LOAD, a `credential_chain` secret, and `s3://` paths
   with the pinned glob `raw/dt=*/*.json*` — which matches *both* key
   shapes deploy day will create: sync's `events.jsonl` files and
   ingest's `evt-*.json` objects, legal because every object body is
   newline-delimited JSON).

   Then the one that closes the sitting: query 08, the reconciliation
   identity, and 003 R10's lesson resurfacing at Wave 2 — **reconcile,
   don't naively compare**. The standing queries read the process
   partition and the traces partition *apart*; your walker reads the
   whole lake; the identity is
   `process_events + trace_lines = glake stats total`. Run both sides
   **back-to-back** — this matters, and here's the proof: at the
   reference's measurement instant (2026-07-10T23:24Z) the identity read
   **350 + 64 = 414**; when these materials were *verified* barely an
   hour later it read (verified):

   ```
   ┌────────────────┬────────────┬────────────────┬────────────────┐
   │ process_events │ trace_rows │ trace_newlines │ identity_total │
   ├────────────────┼────────────┼────────────────┼────────────────┤
   │            354 │         64 │             64 │            418 │
   └────────────────┴────────────┴────────────────┴────────────────┘

   $ ./target/debug/glake stats datalake/raw-local     # same instant
   6 files · 418 events
   ```

   Same lake, same queries, four more events — **the hooks were
   appending telemetry about this course while the course ran**. Both
   instants hold their identity exactly (350+64=414, 354+64=418); a
   count without its instant holds nothing. Your numbers will be
   different again, and must reconcile *against each other*, not against
   this page. Record yours with a timestamp; that habit is what sitting
   U's evidence template is built around (the S12 identity gains a term
   there: + events curl-ingested since the sync). While you're in the
   pack, two DuckDB dialect potholes the reference already hit so you
   can step over them knowingly: `->>` binds looser than comparison, so
   a JSON filter in a WHERE conjunction needs its parens —
   `(payload->>'k') = 'v'`; and a SELECT-list alias whose expression
   contains a subquery can't be lateral-referenced, hence 08's CTE
   shape. **Commit point:**

   ```console
   git add specs/007-lake-to-s3/evidence.md
   git commit -m "007: sitting T — scan pack run, identity reconciled with instant"
   ```

## Fights to expect

One compiler fight, then reading fights — ledger the real ones
(`learning.*`).

- **The Fn-capture wall** — move 3's centerpiece, quoted there in full.
  The note line (`closure implements Fn, so references to captured
  variables can't escape`) is the diagnostic to memorize: it fires
  whenever a service-shaped closure tries to lend its captures to the
  futures it returns. The `OnceLock`-minted `&'static` is the clean way
  out; you'll see `Box::leak` in the wild doing the same job with less
  honesty about intent.
- **`error[E0038]` reprise** — reaching for `Box<dyn ObjectStore>` in the
  handler signature out of `Emit` habit. R's fights section has the full
  text; the generic `handle_with<S>` is the only shape on offer, and it's
  the better one anyway (monomorphized, testable with the fake).
- **The old tests won't compile, and that's information** — every H1
  test that named `Emit` or a Vec sink breaks structurally. Don't
  scaffold compatibility shims; re-aim the observations at the ledger
  (move 4). A seam swap that left the old tests compiling would mean the
  seam wasn't really swapped.
- **The nag-ack round-trip trap** — if your review drove edits and a new
  finding appeared, remember move 6's wart before trusting any suggested
  suppression string: real IDs from real discovery-synth output only,
  and granular ids with `::` must travel by the metadata key.
- **The moving lake** — an identity that misses by a few events was
  almost certainly measured across a gap (a hook fired between the query
  and the walk). Re-run both sides back-to-back before suspecting the
  queries; suspect the *instant* first. (If it misses by exactly the
  trace-line count, you've compared instead of reconciled — R10's
  original sin.)

## Checkpoint

From the repo root — the sitting counts as done only when all of these
hold:

```
cargo fmt --check                                          # no diff
cargo clippy -p hello-lambda --all-targets -- -D warnings  # clean
cargo test -p hello-lambda                                 # 13 green — reference census:
                                                           # 5 unit, 7 examples, H4 @ 512
cargo tree -p hello-lambda -e normal | grep -c aws-sdk     # 4 (was 0 — the supersession, visible)
ls -l target/lambda/hello-lambda/bootstrap                 # ~11.4 MB (reference: 11,431,848 —
                                                           # the bill's last row, filled)
cd infra && npx tsc --noEmit && npx cdk synth              # exit 0, both stacks
jq '.pluginReports' cdk.out/validation-report.json         # []
```

- The stateful template read is done with your own eyes: Retain ×2 per
  bucket · AES256 · BPA all-true · no VersioningConfiguration ·
  terminationProtection true in the manifest · exactly one IAM statement
  (`GoldeneyeLakeRawPutOnly`) · `LAKE_BUCKET` env on the function.
- `specs/006-hello-lambda/requirements.md` carries your two S6b
  changelog rows, in their own commit, awaiting 007's close-gate.
- `evidence.md` holds: the completed S8 bill (three rows, delta, ×6.2,
  bloat top-10, transport-not-client paragraph), your NONE-posture
  re-examination finding, and the scan-pack outputs with the identity
  *and its instant*.
- You can answer aloud: why did `Emit` have to die rather than adapt
  (two reasons — async, and E0038)? Why is a put-failure 500 and not
  202, and which healthz invariant did that weaken? What does the
  `ingest_key` sanitizer protect, and from what? Why does the closure
  need a `&'static` store, and what mints it? What does grantPut bundle
  that the hand-written statement refuses? Which encryption setting is
  secretly a correctness setting, and for which law? And the review
  finding: which written H12 bound went stale today, what changed it,
  and what's your ruling?

## Hints (one at a time)

<details><summary>Hint 1 — the H4 harness surgery, minimal diff</summary>

Three touches and nothing else. (1) `hello_verdict` builds a fresh
`FakeStore` per body and threads it through the generic handler:

```rust
async fn hello_verdict(body: &str) -> Result<(u16, String, Vec<String>), TestCaseError> {
    let store = FakeStore::new();
    let response = handle_with(lambda_post(body), &store).await
        .map_err(|e| TestCaseError::fail(format!("hello handler errored: {e}")))?;
    // …status + text as before…
    Ok((status, text, store.put_log()))
}
```

(2) The emit rider becomes the put rider: `put_keys.len() ==
usize::from(hello_status == 202)`. (3) A key-shape check per put —
`starts_with("raw/dt=")`, `ends_with(".json")`, `matches('/').count()
== 2` — that last clause is the sanitizer's guarantee under test: two
slashes means the object sits exactly where the pinned glob looks. The
relay side, the generator, the `_rx` live-receiver rule, and the domain
bound don't change: the oracle doesn't care what your sink is, which is
the whole point of H4.

</details>

<details><summary>Hint 2 — the scan pack won't run / won't reconcile</summary>

Cheapest first. (1) *`read_json` errors about a path*: you're not at
the repo root — every local query is written relative to it. (2) *08
errors on the traces glob*: a lake with no `traces/` partition yet
makes `read_text` find zero files — your real lake has one; a fixture
lake may not. (3) *The identity misses by a handful*: instant gap —
re-run query-then-walker back-to-back (the fights section's rule). (4)
*The identity misses by exactly `trace_rows`*: you compared glake's
total against `process_events` alone — that's the R10 mistake the
identity exists to prevent; glake walks both partitions. (5)
*`trace_rows != trace_newlines`*: a trace file has a malformed or blank
line — that inequality is itself a finding (the header says why it's
watched); investigate the file before "fixing" the query. And if
duckdb-the-CLI isn't installed, the python module IS the reference
runner: `python3 -c "import duckdb; print(duckdb.sql(open('…').read()))"`.

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours:

- `specs/007-lake-to-s3/_reference/hello-lambda/src/handler.rs` — the
  generic core, `Accepted`, `ingest_key`'s sanitizer comment, and the
  500 branch with its counter note. Path warning as ever: the
  reference's deps point at sibling `_reference` crates; yours are
  `../lake-store` and `../glake`.
- `specs/007-lake-to-s3/_reference/hello-lambda/src/main.rs` — the
  init-once choreography and the `OnceLock` parking, with the cold-start
  argument in its module docs.
- `specs/007-lake-to-s3/_reference/hello-lambda/tests/handler.rs` and
  `tests/prop_door.rs` — the re-aimed suite and the surgical H4 diff.
- `infra/lib/stateful-stack.ts` + `infra/bin/goldeneye.ts` — the review
  targets themselves, comments included (they cite the findings tables
  in `_reference/NOTES.md` §3).
- `specs/007-lake-to-s3/_reference/NOTES.md` §3–§5 — the discovery-synth
  findings, the fourteen decisions (9–12 are this sitting's), and the
  scan-pack outputs with their instant.

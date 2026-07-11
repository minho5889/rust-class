# Sitting S — reality and the bill

**Builds:** lake-sync grows up: the reads go streaming through one reused
line buffer (T4, with an honesty box about what "streaming" really means
here), the uploads get their bound (`JoinSet` + `Arc<Semaphore>`,
`acquire_owned` in the producer loop — proven by the S9 gauge), failure
becomes reportable data (S5: named objects, partial progress, the exit-code
boundary), and the binary arrives (clap, `--dry-run` offline *by
structure*). Then reality: `S3Store`, read together line by line with a full
stop at the etag quote-strip — and **the bill (S8)**: the SDK weighed on a
real ARM64 artifact, next to the 006 baseline you built in P.
**Requirements:** S5, S8, S9 (+S2 binary half; S7 closes) — tasks 1.4–1.5
(T3: the SDK, measured; T4: streaming + buffer reuse).
**Ramp you'll use:** sitting M's channel-ownership lesson (the bounded
buffer returns as a permit pool), N's stdout-is-protocol and exit-code
discipline, F/P's machine-check ritual for the measurement (you drive).

> **Prerequisite (tasks.md):** the ARM64 cross toolchain for the bill's
> lake-sync row. Reference setup: `rustup target add
> aarch64-unknown-linux-gnu` plus a cross linker (`aarch64-linux-gnu-gcc`
> from your distro's `gcc-aarch64-linux-gnu` package) named in
> `crates/lake-sync/.cargo/config.toml`:
>
> ```toml
> [target.aarch64-unknown-linux-gnu]
> linker = "aarch64-linux-gnu-gcc"
> ```
>
> (cargo-lambda + zig, P's prerequisite, covers the *Lambda* artifact — this
> is for the plain binary. In the authoring environment aws-lc-sys compiled
> clean for aarch64 with nothing but that linker line; the design's
> rustls/ring fallback was NOT needed. If your machine disagrees, the fights
> section has the triage.)

## Where you are

R closed with the laws green at 1,024 generated lakes — and with two honest
IOUs in the code: `run` reads whole files with `std::fs::read` and uploads
them all at once, unbounded; and nothing anywhere can fail on purpose except
the fake. Today both IOUs are paid, the CLI makes the tool usable by a
human, and then the seam earns its keep a second way: `S3Store` drops in
behind it, about sixty lines of translation you can read in one pass. The
sitting ends with the number this whole spec was pointed at since P — you
held 1,843,256 bytes in your hand and the guide said *"the calibration point
for the whole spec."* Today you learn what `aws-sdk-s3` actually costs, on a
real artifact, measured by you — and the answer's *shape* (where the bytes
live) matters more than its size.

## The build, move by move

All commands from the repo root. Four commits: streaming, bound+errors,
s3store, the-bill notes — tasks 1.4 and 1.5.

1. **Streaming, honestly (task 1.4 begins).** R's compare reads whole files
   into memory to hash them. Replace it with the discipline this course
   keeps circling (steps 2/4, F's staircases): in `plan.rs`, write
   `md5_of_file(path, line_buffer: &mut String)` — a `BufReader`, a loop of
   `read_line` into the **caller-owned** buffer, each chunk fed to
   `md5::Context::consume`, `clear()` between lines. And `read_body(path,
   line_buffer)` for `run`: same loop, bytes accumulated into the returned
   `Vec<u8>`. One `String` lives at each call site's loop — `plan` re-uses
   it across every size-tied file, `run` across every uploaded file.

   Now the honesty box, out loud, because "streaming" is doing two
   different jobs here and conflating them is how people lie to themselves
   about memory:

   - **The digest truly streams.** Each line passes through `consume` and
     is overwritten by the next; peak memory is one *line*, not one file,
     no matter how big the file grows.
   - **The PUT body accumulates.** A single PUT needs the whole object in
     hand, so `read_body`'s peak is one *file*. The lines stream through
     the reused buffer; the body does not. That's fine *by design* — lake
     files are KB-scale, and multipart is an explicit non-goal — but say
     it, don't hide it.
   - **What reuse buys:** `clear()` resets length, not capacity. After the
     first long line, the buffer stops allocating — the F lens showed you
     what per-line churn looks like; this is the idiom that avoids it.
   - **Two exactness details that make the etag mean anything:**
     `read_line` keeps every byte it read — the `\n`s included, a final
     line *without* one included — so consuming the buffers in order
     digests exactly the file's bytes. (R's generator produced
     trailing-newline-less files on purpose; now you know why.) And the
     cost of the discipline: `read_line` validates UTF-8 — a non-UTF-8
     file surfaces as an io error here instead of syncing. The lake is
     UTF-8 JSONL by construction; refusing loudly beats carrying bytes you
     can't account for.

   Pin it with the byte-exactness unit test: a fixture with a blank line
   and no trailing newline; assert `read_body` reproduces the bytes
   exactly AND that `md5::compute(&body)` equals `md5_of_file`'s answer —
   the digest path and the body path may never disagree. Then the payoff
   of R's test-first: **the laws re-run green over the refactor** (G's
   lesson — a refactor under test is a refactor, anything else is a
   rewrite). **Commit point:**

   ```console
   cargo fmt && cargo clippy -p lake-sync --all-targets -- -D warnings
   cargo test -p lake-sync
   git add crates/lake-sync
   git commit -m "007: sitting S — streaming reads, one reused line buffer"
   ```

2. **The bound (task 1.4, the S9 half).** R's `run` spawns everything at
   once — a 200-file lake would hold 200 bodies in flight. The design says
   ≤ 4, and the shape is 005's bounded-channel lesson re-cast for fan-out:
   one producer (the read loop) feeding N uploads through a bounded
   *permit pool*. Before you write it, the prediction ritual: where must
   the permit be acquired — inside the spawned task, or in the loop before
   spawning? Write your answer and its because down; the compiler referees
   one spelling and the design the other.

   First the compiler's half. The gut spelling — a `Semaphore` on the
   stack, `acquire()` in the loop, permit moved into the task — earns this
   (verified, verbatim):

   ```
   error[E0597]: `semaphore` does not live long enough
      --> src/run.rs:…
       |
       |       let semaphore = Semaphore::new(4);
       |           --------- binding `semaphore` declared here
   ...
       |           let permit = semaphore.acquire().await;
       |                        ^^^^^^^^^ borrowed value does not live long enough
       |           tasks.spawn(async move {
       |               let _permit = permit;
       |               store.put(key, body).await
       |           });
       |           - argument requires that `semaphore` is borrowed for `'static`
   ...
       |   }
       |   - `semaphore` dropped here while still borrowed
       |
   note: requirement that the value outlives `'static` introduced here
       |
   145 |         F: Send + 'static,
       |            ^^^^^^ required by this bound in `JoinSet::<T>::spawn`
   ```

   Same wall as R's, different brick: `acquire()` returns a permit that
   *borrows* the semaphore, and borrows can't enter a `'static` task. The
   Arc-flavored fix is `Arc<Semaphore>` + **`acquire_owned()`** — a permit
   that owns its Arc and can move anywhere. (The `Err` arm of
   `acquire_owned` is unreachable-by-construction — it fails only on a
   closed semaphore and nothing here closes one — but the unwrap ban is
   absolute: degrade it into a named entry in `outcome.failed` instead of
   a panic.)

   Now the design's half, and it's the reason the permit is acquired **in
   the loop, before the read moves into the task**: both spellings bound
   the *puts*, but only this one also bounds **memory** — with 4 permits
   out, the `acquire_owned().await` parks the loop *before it reads body
   #6*, so at most `limit + 1` bodies are ever resident.
   Acquire-inside-the-task would happily read and buffer the whole lake
   into spawned-but-waiting tasks. Write that as a comment in `run.rs`;
   it's the half of S9 the gauge can't see. Inside the task:
   `let _permit = permit;` — and say N's footgun aloud one more time,
   because here it's silent: **`_permit = permit` binds (drops when the
   put resolves, releasing the slot); a bare `_ = permit` drops
   immediately and unbounds the whole thing with no diagnostic.**

   The S9 example test (`tests/cli.rs`): a 24-file generated lake, all
   uploads fresh, then two assertions on the fake's gauge:
   `high_water <= 4` (the law) **and `high_water >= 2`** (the measurement
   is alive). The lower bound is what R's dwell was for: without it, a
   serial (bugged) runner — or a fake whose put never awaits — would pass
   the ≤ 4 check vacuously with a high-water of 1. A bound you can't
   watch being approached isn't a bound; it's a hope.

3. **Failure becomes data (task 1.4 closes — S5).** Two different kinds of
   "wrong" deserve two different exits, and the boundary is *when* the
   problem happens:

   - **Couldn't even try** — bad usage, missing/unreadable path, a LIST
     refusal, an io error mid-read: one stderr line
     (`lake-sync: <message>`, path context included — glake's discipline,
     verbatim), **exit 2**.
   - **Tried and some puts died** — that's not an error, that's an
     *outcome*: the run completes, every failed object is **named** on
     stderr, every uploaded object is reported (partial progress stated,
     not hidden), **exit 1**.

   Give `Outcome` its reporting methods: `stdout_report()` — quiet on
   success (exactly `uploaded N, skipped M`), loud on failure (every
   uploaded key enumerated first, then `uploaded N, skipped M, FAILED n`);
   `stderr_report()` — one `lake-sync: cannot put object …` line per
   failure; `exit_code()` — 0 or 1. One deliberate absence to notice: the
   requirements' example transcript once decorated the plan with an event
   count, and the reference dropped it on purpose — counting events means
   *parsing* lake content, and sync moves **bytes, not judgments** (R's
   generator was built on that rule). glake counts events; lake-sync
   carries them.

   The S5 example test: `fail_on` one key of the three-file fixture lake,
   run, then assert the whole story — `failed` names exactly the doomed
   key, `exit_code() != 0`, stderr contains the key, stdout enumerates
   both survivors AND ends `uploaded 2, skipped 0, FAILED 1`, and the
   store agrees (survivors present, doomed absent). **Commit point:**

   ```console
   cargo fmt && cargo clippy -p lake-sync --all-targets -- -D warnings
   git add crates/lake-sync
   git commit -m "007: sitting S — bounded fan-out (S9 gauge) + S5 failure reporting"
   ```

4. **The binary (task 1.5 begins).** clap, same derive discipline as
   glake: `--bucket` (required **unless** `--dry-run` —
   `required_unless_present`, and the no-unwrap spelling of "clap
   guaranteed it" is a `let Some(bucket) = … else { … }` with a named
   error, defense costs two lines), `--dry-run`, `--prefix` (default
   `raw/`), and the lake path. `main` is the only file that knows AWS
   exists, and its structure IS S2:

   - **The dry-run path constructs no client, loads no credentials, and
     plans against an EMPTY remote listing.** It answers "what does the
     walker see and where would it go?" — a question that needs no
     network. `--bucket` is accepted and ignored (deploy-day muscle
     memory keeps the flag). Say the honest consequence aloud and write
     it in the module docs: a dry-run plan is the **first-sync upper
     bound** — it cannot know what a populated bucket would skip.
   - The real path is the only impure dozen lines in the crate:
     `aws_config::load_defaults(BehaviorVersion::latest()).await` — the
     same env → profile → IMDS chain every AWS tool you've ever
     supported speaks — then ONE client, ONE `list`, and everything else
     is R's pure core. (Add `aws-config = "1"` to lake-sync's
     dependencies now; `lake-store`'s SDK arrives in move 5.)

   Render the plan as `upload <key> (<size> bytes)` / `skip <key>` lines
   plus `plan: upload N, skip M`. Then the binary tests
   (`CARGO_BIN_EXE_lake-sync`), and the house trick that makes S2's
   "no AWS" claim a *proof* instead of a promise: the spawned command gets
   **`env_clear()`** — no `AWS_*` variables, no `HOME`, no nothing. A
   dry-run that passes in a scrubbed environment structurally cannot have
   touched a credential chain. Pin: no args → clap usage on stderr, exit
   2; path without `--bucket` → exit 2 naming the missing flag;
   `--dry-run /no/such/lake` → ONE stderr line with the path, exit 2,
   stdout empty; the fixture-lake dry-run → all three S1 keys verbatim
   (`dt=`, `dt=bad-ts`, `traces/`), `plan: upload 3, skip 0`, run twice →
   identical output, lake untouched; `--prefix raw` ≡ `--prefix raw/`.
   Real outputs to expect (verified against the reference binary):

   ```console
   $ lake-sync --dry-run /no/such/lake; echo "exit=$?"
   lake-sync: cannot read /no/such/lake: No such file or directory (os error 2)
   exit=2

   $ lake-sync /some/lake; echo "exit=$?"        # path but no --bucket
   error: the following required arguments were not provided:
     --bucket <BUCKET>

   Usage: lake-sync --bucket <BUCKET> <PATH>
   exit=2
   ```

   And the first full-scale run of your own tool — against the real lake
   (reference transcript from validation, 2026-07-10; **your numbers will
   differ — the lake grows while you work**, hold that thought for
   sitting T):

   ```console
   $ cargo run -p lake-sync -- --bucket goldeneye-lake --dry-run datalake/raw-local
   upload raw/dt=2026-07-05/events.jsonl (73771 bytes)
   upload raw/dt=2026-07-06/events.jsonl (1000 bytes)
   upload raw/dt=2026-07-08/events.jsonl (1001 bytes)
   upload raw/dt=2026-07-09/events.jsonl (33842 bytes)
   upload raw/dt=2026-07-10/events.jsonl (28412 bytes)
   upload raw/traces/dt=2026-07-05/02-collections-lens-28630.jsonl (14483 bytes)
   plan: upload 6, skip 0
   ```

   Note what you're looking at: the `traces/` file mapped verbatim, your
   whole telemetry history planned for upload, zero credentials in the
   room. This exact command is deploy day's first move (sitting U).

5. **`S3Store` — read the real thing together (task 1.5, T3).** Add the
   real half of lake-store's feature table —

   ```toml
   default = ["s3"]
   s3 = ["dep:aws-sdk-s3"]
   ```

   with `aws-sdk-s3 = { version = "1", optional = true }`, and
   `#[cfg(feature = "s3")] pub mod s3;`. This module is the ONLY place in
   the project that names the SDK (S7) — extended even to the dependency
   graph: re-export `pub use aws_sdk_s3::Client as S3Client;` so
   consumers build a client without naming `aws-sdk-s3` in their own
   manifests. The whole module is ~60 lines of code (~120 with its
   teaching comments); we read it line by line, and you stop me anywhere
   the SDK's shape surprises you. Three full stops are mandatory:

   - **The paginator.** `list_objects_v2` answers at most 1000 keys per
     page; `.into_paginator().send()` turns continuation tokens into a
     stream of pages, drained in a `while let Some(page) =
     pages.next().await` — so callers of the seam never see pagination at
     all. (The seam's `list` contract — "everything under the prefix, in
     one shot" — is *manufactured here*, not given by S3.)
   - **THE QUOTE-STRIP.** S3 returns the ETag **wrapped in literal double
     quotes** — `"9bb58f26192e4ba00f01e2e7b136bbd8"` — because HTTP's
     ETag header is a quoted-string. Now trace the failure that happens
     if you forget to strip them, all the way to the user: `list` yields
     `"\"9bb5…\""`, plan compares it against your computed bare-hex md5,
     **every etag check fails, every size-tied file re-uploads, every
     run** — and nothing errors. `uploaded 0` silently becomes
     `uploaded everything, forever`, wasteful-never-lossy, and the S4
     property can't catch it because the fake speaks bare hex on both
     sides. One `trim_matches('"')`, one comment saying why, at the one
     line where a translation layer this thin can still lie.
   - **Where "etag = md5" holds — the honest paragraph.** Your compare
     treats the etag as the body's md5. That's *guaranteed* for a
     single-part PUT under SSE-S3 — exactly what this project does (KB
     bodies, and the S10 stack you review next sitting mandates SSE-S3).
     It **breaks** under SSE-KMS (etag ≠ body md5) and multipart
     (`md5-of-part-md5s-<count>`). If either ever arrives, the compare
     degrades *safely* — etags stop matching, files re-upload, nothing is
     lost — and the fix would be a different compare, not a different
     law. This is why the stateful stack's encryption choice is a
     *correctness* decision, not just a security one; remember this
     paragraph when you review it in T.

   `put` is a passthrough: `put_object().bucket(…).key(…).body(ByteStream
   ::from(body)).send().await`, error mapped with the key attached.
   That's the whole module — everything the properties couldn't cover is
   thin enough for review + compile + deploy day to carry. Wire `main`'s
   real path to `Arc::new(S3Store::new(S3Client::new(&config), bucket))`,
   full gate, **commit point:**

   ```console
   cargo fmt
   cargo clippy -p lake-store --all-targets --all-features -- -D warnings
   cargo clippy -p lake-sync --all-targets -- -D warnings
   cargo test -p lake-sync                    # full suite now: 5 + 8 + 2
   git add crates/lake-store crates/lake-sync Cargo.lock
   git commit -m "007: sitting S — S3Store behind the seam, clap CLI, dry-run offline"
   ```

6. **The bill (task 1.5 closes — S8, you drive).** P made you hold the
   number: **1,843,256 bytes**, the no-SDK baseline, "what 007 gets
   measured against." Collect. First, predictions in writing (the ritual
   never changes): the constitution says `aws-sdk-*` adds "~10 MB+" —
   where do you think those megabytes *live*? Rank your guesses: the S3
   API surface? TLS? HTTP? Retry/middleware machinery? Then measure.

   ```console
   ls -l target/lambda/hello-lambda/bootstrap     # P's artifact — the baseline row
   cargo build -p lake-sync --release --target aarch64-unknown-linux-gnu
   ls -l target/aarch64-unknown-linux-gnu/release/lake-sync
   ```

   Reference numbers (validated; same profile flags as your workspace
   root — same flags or the number lies):

   | Artifact | Bytes | MiB |
   |---|---|---|
   | 006 `bootstrap` (P's baseline, no SDK) | 1,843,256 | 1.76 |
   | lake-sync, ARM64 release | **11,435,352** | **10.90** |

   Same order of magnitude as the whole warning. Now *where it lives* —
   the bloat read (host build, ranking-not-bytes is the durable part, P's
   caveat):

   ```console
   cargo bloat --release --crates -n 10 -p lake-sync
   ```

   Reference output (verified):

   ```
    File  .text     Size Crate
    9.2%  20.0%   1.8MiB std
    6.6%  14.4%   1.3MiB aws_lc_sys
    2.8%   6.2% 557.0KiB [Unknown]
    2.7%   5.9% 529.2KiB rustls
    2.5%   5.3% 480.6KiB h2
    1.9%   4.1% 368.9KiB aws_smithy_runtime
    1.8%   3.9% 350.5KiB aws_sdk_s3
    1.5%   3.2% 290.6KiB aws_config
    1.4%   3.0% 269.6KiB hyper
    1.2%   2.6% 236.3KiB ring
   ```

   Read it against your prediction, because this is the sitting's
   punchline and the through-line's newest entry: **you pay for the
   transport, not the client.** aws_lc_sys (AWS-LC crypto, C, pregenerated
   bindings) + rustls + ring ≈ 2 MiB of TLS/crypto; h2 + the hyper growth
   ≈ another 1½ MiB of HTTP/2 (S3 speaks h2 — 006's runtime-API client
   was h1, which is why P's table never showed it); the smithy runtime is
   the SDK's client machinery — while **`aws_sdk_s3` itself, the thing you
   actually asked for, is ~350 KiB**. Sixty lines of your code, ~350 KiB
   of API surface, and nine megabytes of production-grade encrypted
   transport underneath it. Two crypto libraries riding together (aws-lc
   AND ring, via rustls's default feature set) is a known SDK artifact —
   a future size hunt could unify them; note it, don't chase it today.
   Also there and worth naming: clap rides in *this* binary and never in
   the Lambda — which is exactly why the three-crate layout exists (the
   design's S8 de-confounding decision, now visible in a table).

   Draft the S8 evidence now (`specs/007-lake-to-s3/evidence.md`, from
   `specs/_template/` if first entry): the two rows above, your top-10,
   the punchline paragraph in your own words — and a **third row left
   deliberately empty**: the evolved ingest Lambda doesn't exist until
   sitting T's sink swap. Write your predicted size in its cell. (The
   validated reference's answer, for calibration when you get there:
   **11,431,848 bytes — +9,588,592 over the baseline, +9.14 MiB, ×6.2**
   — within a rounding error of lake-sync's number, because the SDK
   dominates both and clap was never in the Lambda's tree.) The
   constitution's "~10 MB+" survives contact with reality; say so in the
   draft, per S8's "if the delta departs wildly, explain" clause.

   Last line of the ledger — the Graviton checklist (S7): the only
   hash/crypto **your code** performs is md5 via the pure-Rust `md5`
   crate, a compatibility checksum, not security. No hardware backend to
   verify: **n/a by construction, recorded** (the sha2 4–5× lesson stays
   armed for the day a real crypto crate arrives; the SDK's TLS uses
   aws-lc's own aarch64 paths — its business, not your checksum's).
   **Commit point:**

   ```console
   git add crates/lake-sync specs/007-lake-to-s3/evidence.md
   git commit -m "007: sitting S — the bill: SDK weighed on ARM64, transport-not-client"
   ```

## Compiler fights to expect

Ledger them (`learning.*`). Today's set is small but two are silent —
the worst kind.

- **`error[E0597]: `semaphore` does not live long enough`** — move 2's
  centerpiece, quoted there in full. The chain to internalize:
  `JoinSet::spawn` demands `'static` → the permit from `acquire()` borrows
  the semaphore → a borrow can't outlive its owner into a spawned task.
  `Arc<Semaphore>` + `acquire_owned()` is the tokio-blessed spelling; note
  the *pattern* (Arc-flavored variants of borrowing APIs exist precisely
  for spawn boundaries) — you'll meet `_owned` suffixes again.
- **The silent unbinding: `_ = permit`.** No diagnostic, no test failure
  in small lakes — just a bound that isn't. The S9 gauge test is the
  tripwire (high-water climbs past 4 on the 24-file lake); if you see
  `high_water: 24 > 4`, check the underscore first. N's footgun, third
  appearance: `_x` binds, `_` drops, and this time dropping is *quiet*.
- **The vacuous gauge.** If your S9 test passes with `high_water == 1`,
  nothing overlapped — usually a `.await` on the spawn result (turning
  fan-out into a serial loop) or a fake whose dwell you removed. The
  `>= 2` assertion exists to make this a red, not a shrug.
- **Cross-build failures at `aws-lc-sys` or link time** — if the ARM64
  build errors mention `cc`, `crt` objects, or a linker it can't find,
  it's toolchain, not you: the `.cargo/config.toml` linker line is
  missing or the cross-gcc isn't installed. In the authoring environment
  the linker line alone sufficed; if aws-lc-sys itself refuses to
  cross-compile on your machine, the design's documented fallback is the
  SDK's rustls/ring feature set — a manifest change, not a code change.
- **`grep -c` exits 1 on zero matches** — F's paper cut, still armed in
  any `set -e` script around today's tree checks.

## Checkpoint

From the repo root — the sitting counts as done only when all of these
hold:

```
cargo fmt --check                                            # no diff
cargo clippy -p lake-store --all-targets --all-features -- -D warnings
cargo clippy -p lake-sync --all-targets -- -D warnings       # both clean
cargo test -p lake-sync                                      # 15 green — reference census:
                                                             # 5 unit + 8 examples + 2 properties @ 512
cargo run -p lake-sync -- --bucket goldeneye-lake --dry-run datalake/raw-local
                                                             # your whole lake planned, traces/ verbatim,
                                                             # plan: upload N, skip 0 — zero credentials
ls -l target/aarch64-unknown-linux-gnu/release/lake-sync     # ~11.4 MB (reference: 11,435,352)
cargo tree -p lake-sync -e normal | grep -c aws-sdk          # 4 — s3 + aws-config's providers
                                                             # (sso, ssooidc, sts ride along; expect
                                                             # the family, not just the one you named)
cargo tree -p hello-lambda -e normal | grep -c aws-sdk       # 0 — the baseline is STILL clean
                                                             # (the swap is sitting T; grep -c
                                                             # prints 0 AND exits 1)
```

Reference suite transcript for the new files (verified — names and
counts):

```
running 8 tests
test s2_dry_run_flow_shows_zero_puts_on_the_ledger ... ok
test s5_fail_on_mid_run_names_object_reports_partial_progress ... ok
test s5_no_args_is_usage_exit_2 ... ok
test s5_missing_bucket_without_dry_run_is_usage_exit_2 ... ok
test s1_prefix_is_normalized ... ok
test s9_gauge_proves_the_concurrency_bound ... ok
test s5_unreadable_path_one_stderr_line_exit_2 ... ok
test s2_dry_run_plans_zero_puts_no_aws ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

- `git log --oneline -4` shows streaming → bound+errors → s3store →
  the-bill notes.
- `evidence.md` holds the S8 draft: both measured rows, your top-10, the
  transport-not-client paragraph, the empty ingest row with your
  prediction, and the Graviton n/a-by-construction note.
- You can answer aloud: what streams and what accumulates in your reads,
  and what is the peak memory of each path? Why does the permit get
  acquired in the loop and not in the task — which bound does each
  spelling enforce? Why does the S9 test assert a *lower* bound on the
  high-water mark? What exits 2 and what exits 1, and what's the rule
  that divides them? What dies without the quote-strip, and why can't
  the S4 property catch it? Under exactly which two conditions does
  etag = md5 hold — and which sitting-T review item guards them? And
  the bill: where do the nine megabytes live, and what would you tell a
  Lambda design review that asks "how much does adding S3 cost the
  binary?"

## Hints (one at a time)

<details><summary>Hint 1 — run's bounded loop, the exact choreography</summary>

```rust
let semaphore = Arc::new(Semaphore::new(MAX_IN_FLIGHT));
let mut tasks: JoinSet<(String, Result<(), StoreError>)> = JoinSet::new();
let mut line_buffer = String::new();

for Upload { key, path, .. } in plan.upload {
    let body = read_body(&path, &mut line_buffer)?;          // sequential: buffer reused
    let permit = match Arc::clone(&semaphore).acquire_owned().await {
        Ok(permit) => permit,                                 // ← parks HERE at 4 in flight
        Err(closed) => { /* named entry in outcome.failed; continue */ }
    };
    let store = Arc::clone(&store);
    tasks.spawn(async move {
        let _permit = permit;                                 // binds; drops when put resolves
        let result = store.put(key.clone(), body).await;
        (key, result)
    });
}
while let Some(joined) = tasks.join_next().await { /* Ok((key, Ok)) / Ok((key, Err)) / Err(join) */ }
```

The `?` on `read_body` is the exit-2 path (an io error mid-run follows
the io discipline — in-flight uploads may be cancelled by the early
return, which is safe *precisely because sync is idempotent*: the next
run heals). The `JoinSet`'s payload carries the key alongside the result
so failures stay matchable without parsing messages.

</details>

<details><summary>Hint 2 — the ARM64 build won't link (triage order)</summary>

Work cheapest-first. (1) `rustup target list --installed` — is
`aarch64-unknown-linux-gnu` there? (2) `which aarch64-linux-gnu-gcc` —
is the cross-gcc installed, and does
`crates/lake-sync/.cargo/config.toml` name it? (`.cargo/config.toml` is
directory-scoped: it must sit in the crate you're building from, or at
the workspace root.) (3) If the error names `aws_lc_sys` and a C
compile (not link) failure, set `CC_aarch64_unknown_linux_gnu` to the
cross-gcc and retry; if it still refuses, take the design's documented
fallback — switch the SDK to its rustls/ring feature set in
lake-store's manifest — and note the deviation in evidence. (4) A build
that succeeds but produces an x86-64 binary means the `--target` flag
went missing: `file` the artifact, always — P's habit. None of these
are Rust fights; they're the CLAUDE.md cross-compile rule collecting
its toll without Docker.

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours:

- `specs/007-lake-to-s3/_reference/lake-sync/src/run.rs` — the permit
  choreography with the memory-bound comment, and the Outcome reports.
- `specs/007-lake-to-s3/_reference/lake-sync/src/plan.rs` — `md5_of_file`
  and `read_body` with the honesty-box comments.
- `specs/007-lake-to-s3/_reference/lake-sync/src/main.rs` — the
  dry-run-is-offline module docs and the exit-code boundary.
- `specs/007-lake-to-s3/_reference/lake-store/src/s3.rs` — the paginator,
  the quote-strip comment, and the etag honesty paragraph, exactly where
  the design pinned them.
- `specs/007-lake-to-s3/_reference/lake-sync/tests/cli.rs` — the
  `env_clear` helper, the S5/S9 library tests, and the binary examples.
- `specs/007-lake-to-s3/_reference/NOTES.md` §1 — the full bill: both
  ARM64 builds, the section arithmetic, the bloat tables, and the
  cross-build note.

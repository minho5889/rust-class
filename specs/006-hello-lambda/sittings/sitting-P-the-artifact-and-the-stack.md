# Sitting P — the artifact and the stack

**Builds:** nothing new that computes — this sitting produces the **real
deployable artifact** (`cargo lambda build --release --arm64`, sized, bloated,
and read against the workspace release profile) and reviews the **real CDK
stack** Claude wrote (`infra/`), you sitting as the AWS professional: the
AuthType=NONE security review, the synth template read, and the three cdk-nag
acknowledgments — including the finding that the nag rule you'd expect
*doesn't exist*.
**Requirements:** H6–H8, H12 — tasks 1.3–1.4 (T2: bootstrap; T3: the release
profile in anger).
**Ramp you'll use:** sitting F's machine-check ritual (Claude drives, you
interrogate — today it flips: *you* drive the build, and you *review* the
TS), and sitting N's tree discipline.

> **Prerequisite (tasks.md):** `cargo lambda` and a zig toolchain installed
> locally — `cdk synth` bundles the function by invoking them. Reference
> toolchain: cargo-lambda 1.9.1 with zig via the `ziglang` Python package
> (`pip install cargo-lambda` or `brew install cargo-lambda` / `cargo install
> cargo-lambda`, then `pip install ziglang` if `zig` isn't already on PATH —
> cargo-lambda discovers the Python package by itself). Check before
> starting: `cargo lambda --version`.

## Where you are

Sitting O ended with a handler proven equivalent to relay's door — 10 tests,
H4 at 512 cases — but "proven" so far means *on your machine, in a test
harness*. Today the crate becomes the thing Lambda actually runs: a single
ARM64 executable named `bootstrap`. Then the division of labor flips the F
way, twice. Task 1.3 you drive: build the artifact, weigh it, and read what's
inside — this move IS the course's memory-management through-line wearing
deployment clothes (no GC, no runtime, a binary you can account for byte by
byte, and in Q, a cold start that's mostly "how fast does Linux exec 1.8 MB").
Task 1.4, Claude drove: months of CDK API churn absorbed so you don't have to
— but the stack deploys to *your* account in sitting Q, so you review it
end-to-end as what you are: the person in the room who has actually run
security reviews on other people's serverless stacks. If the written AuthType=NONE justification
doesn't convince you, we amend design.md before anything deploys — that's the
change protocol, not a courtesy.

## The build, move by move

All commands from the repo root unless noted. Two commits: build notes, then
any review-driven infra edits (or the review note itself).

1. **Build the artifact (task 1.3, you drive).**

   ```console
   cargo lambda build --release --arm64 -p hello-lambda
   ls -l target/lambda/hello-lambda/bootstrap
   file target/lambda/hello-lambda/bootstrap
   ```

   Reference numbers (validated during authoring — your workspace build
   shares the same profile flags but a different `Cargo.lock`, so expect
   the same band, not the same byte):

   ```
   -rwxr-xr-x 1 … 1843256 … bootstrap
   bootstrap: ELF 64-bit LSB pie executable, ARM aarch64, dynamically linked,
   interpreter /lib/ld-linux-aarch64.so.1, for GNU/Linux 2.0.0, stripped
   ```

   **1,843,256 bytes = 1.76 MiB.** Hold that number; it's the calibration
   point for the whole spec — the no-SDK baseline 007 gets measured
   against (007's first lesson will be watching `aws-sdk-*` add ~10 MB to
   this). Three things to read off the `file` line as the AWS pro:

   - `ARM aarch64` — the artifact is Graviton-native (H6). No Docker, no
     QEMU: cargo-lambda cross-compiled via zig from whatever host you're
     on. This is the CLAUDE.md cross-compile rule with the pain removed.
   - `dynamically linked` — and that's *fine*, not a broken static-linking
     story: zig linked against glibc-compatible symbols that
     `provided.al2023` guarantees. The T3 cold-start argument was never
     about static-vs-dynamic libc — it's about what the binary *doesn't
     carry*: no JVM, no interpreter, no GC, no JIT. What you see is all
     there is.
   - `stripped` — the workspace release profile's `strip = "symbols"`
     doing its job (debuginfo has been auto-stripped since 1.77; symbols
     are the part you have to ask for).

   The name `bootstrap` is the entire custom-runtime contract (T2):
   `provided.al2023` boots, execs the file by that name, and your binary
   speaks the Runtime API from there. You've read that sentence in AWS
   docs a hundred times — the difference today is that every byte of the
   thing being exec'd is accounted for in *your* Cargo.lock.

2. **Weigh what's inside — the bloat read (task 1.3, still you).** H6
   wants the size *and* the top-10 recorded. `cargo install cargo-bloat`
   if you don't have it, then:

   ```console
   cargo bloat --release --crates -n 10 -p hello-lambda
   ```

   One honesty note first: bloat can't cross-compile, so this analyzes
   the **host** build — design.md records it as an approximation, and the
   ranking (not the exact KiB) is the durable read. Reference output:

   ```
    File  .text     Size Crate
   12.3%  39.8% 519.9KiB std
    2.0%   6.4%  83.7KiB serde_json
    1.9%   6.2%  81.4KiB tokio
    1.9%   6.1%  80.0KiB hyper_util
    1.7%   5.5%  71.9KiB http
    1.5%   4.9%  63.9KiB hyper
    1.2%   3.7%  48.9KiB lambda_runtime
    1.0%   3.2%  42.1KiB url
    0.8%   2.7%  34.7KiB encoding_rs
    0.7%   2.2%  29.1KiB idna
   30.8% 100.0%   1.3MiB .text section size
   ```

   Now the reading, and this table is the memory-management through-line
   in one screen — spend real minutes on it:

   - **std at 39.8%** is the price of a real standard library compiled
     into the binary (no shared runtime to lean on — the binary IS the
     runtime, so it carries its own allocator, formatter, unwinder-less
     panic machinery, collections).
   - **The hyper/http/url/encoding_rs/idna chain plus lambda_runtime** —
     add them up: that's *the HTTP client that talks to the Lambda
     Runtime API*. The "server you don't write" from sitting O isn't
     free; it's just small and visible. The runtime loop you deleted from
     relay's `main.rs` didn't vanish — it moved into ~250 KiB of
     someone-else's-audited Rust.
   - **Where are glake and your handler?** Not in the top 10.
     Noise-level. The thing you spent two sittings writing costs less
     than the URL parser. Sit with that: idiomatic Rust application code
     compiles down to almost nothing; the platform-speaking machinery
     dominates — and the *whole thing* is under 2 MiB.
   - **The tradeoffs you can now audit:** sitting O's decision to
     hand-roll the `started` clock instead of adding a date crate, and to
     feature-pick `lambda_http` down to `apigw_http` — this table is
     what those decisions were defending. Every crate you add buys a row
     here, and Q will show you what rows cost at cold start.

   Then tie each release-profile flag to what it bought (the flags are in
   the workspace root `Cargo.toml`, with comments — the reference
   replicated them verbatim so its numbers are honest for your crate):
   `lto = "thin"` (cross-crate inlining and dead-code elimination — the
   reason unused lambda_http codecs actually disappear), `codegen-units
   = 1` (whole-crate optimization; slower compile, better and smaller
   code — fine for deployables), `panic = "abort"` (no unwind tables or
   landing pads in the binary; on Lambda an aborted process is a failed
   invocation and a fresh instance — the platform is your unwinder), and
   `strip = "symbols"` (move 1's `stripped`). If you want the section
   arithmetic: `llvm-size target/lambda/hello-lambda/bootstrap` (plain
   binutils `size` may balk at a foreign-arch ELF) on the reference
   artifact reports text 1,767,224 · data 72,984 · bss 5,208 —
   the binary is ~96% code, almost no static data, and a 5 KB
   zero-initialized tail. **Commit point** (H6's evidence draft rides
   with the crate):

   ```console
   git add crates/hello-lambda specs/006-hello-lambda/evidence.md
   git commit -m "006: sitting P — artifact built: 1.8 MB arm64 bootstrap, bloat read"
   ```

   (Start `evidence.md` from `specs/_template/` if this is its first
   entry: record your bootstrap size, your top-10, and one sentence per
   profile flag. Q appends the deploy-day numbers to the same file.)

3. **Meet the stack (task 1.4 — Claude wrote it, you review it).** The
   `infra/` app is born (H8): CDK v2 TypeScript, two stacks — stateless
   (this spec, freely destroyable) and stateful (a documented stub 007
   fills; deliberately **not instantiated**, so nothing 006 deploys or
   destroys can touch future lake state by accident). Read in this order,
   end-to-end — it's ~200 lines total:

   1. `infra/README.md` — the two synth context modes and the auth
      decision summary;
   2. `infra/bin/goldeneye.ts` — app-wide wiring: the `project=goldeneye`
      tag on everything (tags over names, per the constitution), cdk-nag
      v3 registered through CDK-native
      `Validations.of(app).addPlugins(new AwsSolutionsChecks(app), new
      ServerlessChecks(app))` — both packs CLAUDE.md requires; note it's
      NOT the Aspects-based v2 API most blog snippets still show —
      and `goldeneye-stateless` pinned to us-east-1 but account-agnostic;
   3. `infra/lib/stateless-stack.ts` — the review target. Six things to
      interrogate, pro to pro:

   - **`RustFunction`** (cargo-lambda-cdk): `manifestPath` resolves the
     `helloLambdaDir` context key, default `../crates/hello-lambda` —
     *your* crate. Synth shells out to the same local cargo-lambda you
     used in move 1; that's why the toolchain is this sitting's
     prerequisite.
   - **`binaryName: 'hello-lambda'`** — a quirk worth knowing: any
     manifest containing a `[workspace]` table (the reference crate's
     empty opt-out counts) makes cargo-lambda-cdk demand a binary name.
     Harmless for your crate; recorded in NOTES.
   - **`architecture: Architecture.ARM_64`, explicit** — cargo-lambda-cdk
     defaults to x86_64, and an architecture/bundling mismatch is a
     *silent runtime failure* (the CLAUDE.md warning, now one line of
     reviewed TypeScript). This line is the reason the bundled asset is
     byte-for-byte the same aarch64 bootstrap you built in move 1.
   - **`memorySize: 128`, `timeout: Duration.seconds(10)`** — 128 MB is
     the H10 experiment's lower setting, not a production judgment; Q
     raises it to 512 as the experiment's second leg.
   - **Two `CfnOutput`s** — `FunctionUrl` (the H9 transcripts) and
     `FunctionName` (fetching REPORT lines in H10). Outputs as the
     deploy-day API between the stack and the sitting.
   - **No log-group resource.** You already know what that means; hold
     the thought for Q's teardown (it's a real gotcha, and the guide will
     collect on it).

4. **The security review: AuthType=NONE (H12 — this is a genuine gate,
   not theater).** The Function URL is public and unauthenticated. The
   full justification is a block comment above the URL in
   `stateless-stack.ts`; it makes four claims. Challenge each one the way
   you'd challenge a customer's:

   1. *Lifetime*: deployed and destroyed in the same sitting (H11), never
      left running unattended. — Is that bound real? What enforces it?
      (Answer: only the spec and you. H11 makes teardown an acceptance
      criterion, and the ops checklist requires "deploy AND teardown both
      logged" — but no technology does. Say that out loud; it's the
      honest shape of the control.)
   2. *Blast radius*: the role is logs-only; the binary ships no aws-sdk
      (you verified the tree yourself in O's checkpoint); it can reach no
      data store. — Cross-check against the synthesized role in move 5.
   3. *Exposure*: worst case, an internet stranger pays us a 400, or
      donates a syntactically-valid JSON line to a log group that dies
      with the stack. — Is there a case the comment misses? (Cost-as-DoS
      is the classic one: a public URL can be hammered. Bound it: 10 s
      timeout × 128 MB × Lambda pricing, and the sitting-long lifetime.
      If that arithmetic doesn't calm you, that's a finding — raise it.)
   4. *Alternative*: AWS_IAM + SigV4-signed curls would consume the
      sitting on request signing instead of the lesson. The IAM variant
      is documented in the same comment for anyone keeping the endpoint
      alive longer.

   Now the finding the course wants you to internalize, because it cuts
   against professional instinct: **you'd expect a guardrail to flag
   this, and none exists.** cdk-nag 3.0.1's AwsSolutions and Serverless
   packs have **no Function-URL-auth rule** — a discovery synth with
   acknowledgments disabled was run during authoring precisely to check,
   and the auth decision therefore carries *no suppression*, because
   there is nothing to suppress. Per the audit directive, no rule ID was
   invented to hang the text on (an invented ID would be worse than
   none: it would *look* machine-checked). The justification lives as
   the stack comment + `infra/README.md` + a NOTES.md record of the
   rule's absence — and if a future cdk-nag grows such a rule, synth
   fails and forces the text into a real acknowledgment. The lesson,
   stated plainly: **a clean nag report is not a clean bill of health —
   know what your linter doesn't check.** Your explicit acknowledgment
   of the NONE posture — in your own words, in the review note — is the
   H12 gate. If the written bound doesn't convince you, we stop here and
   amend design.md before Q.

5. **Synth, and read the template like a diff (task 1.4 closes).**

   ```console
   cd infra && npm install && npx tsc --noEmit   # exit 0 — types check
   npx cdk synth                                  # bundles YOUR crate via cargo-lambda
   ```

   (Synth needs no credentials. The reference-mode variant
   `npm run synth:reference` builds the answer-key crate instead — it's
   what CI/authoring used; both were validated exit 0. If synth dies
   complaining a `Cargo.toml` isn't where it expected, you're missing
   sitting O's crate or the context key — see the fights.)

   Zero findings is the required result (H8): the nag validation report
   at `cdk.out/validation-report.json` must read `"pluginReports": []`.
   Then read what will actually be deployed —
   `cdk.out/goldeneye-stateless.template.json`. You've read a thousand
   of these in tickets; this one is six resources, and every line is
   yours to veto. The jq rounds below were run against the validated
   reference synth (your logical IDs will match; hashes may differ):

   ```console
   jq '.Resources | map_values(.Type)' cdk.out/goldeneye-stateless.template.json
   ```
   ```json
   {
     "HelloLambdaServiceRoleE071F162": "AWS::IAM::Role",
     "HelloLambda3D9C82D6": "AWS::Lambda::Function",
     "HelloLambdaFunctionUrlBB4EB85D": "AWS::Lambda::Url",
     "HelloLambdainvokefunctionurlB58E9CF3": "AWS::Lambda::Permission",
     "HelloLambdainvokefunctionB6AFFFC0": "AWS::Lambda::Permission",
     "CDKMetadata": "AWS::CDK::Metadata"
   }
   ```

   ```console
   jq '.Resources[] | select(.Type=="AWS::Lambda::Function").Properties
       | {Architectures, Runtime, Handler, MemorySize, Timeout, Tags}' \
       cdk.out/goldeneye-stateless.template.json
   ```
   ```json
   {
     "Architectures": ["arm64"],
     "Runtime": "provided.al2023",
     "Handler": "bootstrap",
     "MemorySize": 128,
     "Timeout": 10,
     "Tags": [{ "Key": "project", "Value": "goldeneye" }]
   }
   ```

   Assertions to make with your own eyes (the H8 list): `arm64` (the one
   explicit line in move 3, surviving to CloudFormation),
   `provided.al2023`, `Handler: bootstrap` (a placeholder string on
   OS-only runtimes — the platform execs the file, not the handler name;
   you knew that, but now it's *your* placeholder), the project tag on
   the function AND the role, `AuthType: "NONE"` on the URL — and the two
   permission resources, which repay a closer look:

   ```
   lambda:InvokeFunctionUrl   FunctionUrlAuthType=NONE          Principal=*
   lambda:InvokeFunction      InvokedViaFunctionUrl=true        Principal=*
   ```

   The first is the classic public-URL permission. The second is newer
   CDK being careful on your behalf: a `Principal: *` invoke permission
   **conditioned on `InvokedViaFunctionUrl: true`** — the wildcard can
   only invoke *through the URL*, not `lambda:InvokeFunction` directly.
   Smaller grant than most consoles hand out. Verify the role while
   you're here: exactly one managed policy
   (`…service-role/AWSLambdaBasicExecutionRole`), no inline policies —
   the logs-only claim from move 4, in the artifact. Finally, confirm the
   bundled asset is the same binary you built:
   `file cdk.out/asset.*/bootstrap` → the same aarch64 line (reference:
   byte-identical at 1,843,256).

6. **The three acknowledgments — do the written reasons hold up?** The
   discovery synth (acknowledgments disabled) reported exactly three
   findings; each is acknowledged in `stateless-stack.ts` with a written
   reason. Review them like suppression tickets, because that's what they
   are:

   | Finding | Level | The written reason — your call |
   |---|---|---|
   | `AwsSolutions-IAM4[Policy::arn:<AWS::Partition>:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole]` | Error | "the managed policy grants exactly the logs-only access this function needs; a customer-managed replacement would duplicate the same three log actions with no scope reduction." You know IAM4's intent (managed policies drift, customer-managed scope tighter). For a role whose entire job is three `logs:` actions on its own group — does the reason hold? |
   | `Serverless-LambdaDLQ` | Error | "synchronous Function-URL invocation only; a DLQ would never receive a message." You wrote this sentence in tickets yourself: DLQs serve *async* invocation. Function URLs are request/response. Hold or fold? |
   | `Serverless-LambdaTracing` | Warning | "H10 reads REPORT lines, not X-Ray traces; active tracing adds a sidecar cost and an SDK surface to a function measured for its minimal footprint." Note it's warning-level and was acknowledged *anyway* — house rule: reviewers see zero findings or real ones, never "empty except noise." |

   One implementation wart to know exists (it's a comment block in the
   stack, full write-up in NOTES.md §3): the granular IAM4 id embeds
   `::` inside `[Policy::…]`, and aws-cdk-lib 2.261's
   `Validations.acknowledge()` rejects any id with more than one `::`
   pair — even though the synth output literally suggests acknowledging
   that exact string. That one acknowledgment is therefore recorded via
   `Validations.ACKNOWLEDGED_RULES_METADATA_KEY` — the same public
   channel `acknowledge()` writes and cdk-nag reads. Worth an upstream
   issue; also worth remembering the next time a tool's own suggestion
   won't round-trip through its own API. **Commit point** — any
   review-driven infra edits, or the review note if the stack survived
   intact:

   ```console
   git add infra specs/006-hello-lambda/evidence.md
   git commit -m "006: sitting P — stack reviewed: NONE posture acked, nag reasons hold"
   ```

## Fights to expect

Almost nothing here is a compiler fight — this sitting's fights are
tooling and *reading* fights. Ledger the real ones (`learning.*`).

- **`cargo lambda: command not found` / zig discovery failures** — the
  prerequisite box. cargo-lambda finds zig on PATH, or via the `ziglang`
  Python package; if the build errors mention neither arm64 nor your
  code, it's toolchain, not you.
- **Synth: `'…/crates/hello-lambda/Cargo.toml' is not a path to a Cargo
  Toml file`** — the default `helloLambdaDir` context expects sitting O's
  crate at `crates/hello-lambda`. Before that crate exists (or from a
  stale checkout) this is the *expected* failure — documented in
  `infra/README.md`; the reference-mode context key is the workaround
  that never needs your crate.
- **The `file` reading fight: "dynamically linked — so much for static
  Rust?"** — move 1's answer: the T3 claim was never static libc; it's
  no-runtime-no-GC-no-JIT. Interrogate the interpreter line instead:
  `ld-linux-aarch64.so.1` exists on provided.al2023, which is the whole
  contract.
- **The bloat reading fight: "why is MY code not in the table?"** — it
  is; it's just below the noise floor of the runtime-API client. The
  correct conclusion is not "bloat is broken" but "application logic is
  nearly free; dependencies are the budget" — which is why H7 pins the
  dependency list and why 007 measures the SDK before adopting it.
- **The nag reading fight** — expecting a Function-URL-auth finding and
  concluding, when none appears, that the posture was blessed. Move 4's
  whole point, restated once: the report is silent because *no rule
  exists*, not because the risk was reviewed by a machine. Zero findings
  ≠ zero risks; know your linter's coverage map.
- **`grep -c` exits 1 on zero matches** — F's paper cut, still armed, if
  you script any of today's checks with `set -e`.

## Checkpoint

From the repo root — the sitting counts as done only when all of these
hold:

```
ls -l target/lambda/hello-lambda/bootstrap        # exists; size recorded in evidence.md
                                                  # (reference calibration: 1,843,256 B = 1.76 MiB)
file target/lambda/hello-lambda/bootstrap         # ELF … ARM aarch64 … stripped
cargo bloat --release --crates -n 10 -p hello-lambda
                                                  # top-10 recorded; std ~40%, then the
                                                  # runtime-API client chain; your code absent
cargo tree -p hello-lambda -e normal | grep -c aws-sdk    # 0 — still the no-SDK baseline (H7)
cd infra && npx tsc --noEmit                      # exit 0
npx cdk synth                                     # exit 0, bundles your crate
jq '.pluginReports' cdk.out/validation-report.json # []   (H8: zero unacknowledged findings)
```

- The template read is done with your own eyes: arm64 ·
  provided.al2023 · MemorySize 128 · `AuthType: NONE` · project tag on
  function and role · one managed policy on the role · the two scoped
  permissions · `file cdk.out/asset.*/bootstrap` says aarch64.
- `evidence.md` holds the H6 numbers (size, top-10, one sentence per
  profile flag) and your written H12 acknowledgment of the NONE posture
  — or the design amendment that replaced it.
- `git log --oneline -2` shows the artifact commit and the review commit.
- You can answer aloud: what does each of the four release-profile flags
  buy, and which one is also a *behavioral* choice on this platform
  (panic=abort — the platform replaces the instance; there's nothing to
  unwind *to*)? Why is the binary mostly an HTTP client, and what does
  that say about what "serverless" actually outsources? Why does the
  auth decision carry no nag suppression, and what would make that
  stop being true? And the one for Q: which resource is *missing* from
  the template, and what will that mean after `cdk destroy`?

## Hints (one at a time)

<details><summary>Hint 1 — the review checklist, condensed</summary>

Work `stateless-stack.ts` top to bottom against this list; every line
should land in exactly one bucket — *understood*, *challenged*, or
*flagged*:

```text
context key + manifestPath      → whose crate builds? (yours, by default)
binaryName                      → the [workspace] quirk (NOTES §3)
Architecture.ARM_64             → explicit or silent-x86 (CLAUDE.md rule)
memorySize / timeout            → H10's lower setting / blast-radius bound
addFunctionUrl(NONE)            → the H12 review target
two CfnOutputs                  → deploy-day API (H9 URL, H10 name)
metadata-key IAM4 ack           → the :: rough edge (why not acknowledge()?)
acknowledge(DLQ, Tracing)       → do the reasons hold? (move 6 table)
the NONE comment block          → four bounds — challenge each
what's NOT there                → log group; stateful stack not instantiated
```

If a line lands in no bucket, that's the line to bring to session.

</details>

<details><summary>Hint 2 — synth produces findings for you (or won't run)</summary>

Three separate smells. (1) *Findings appear*: you edited the stack during
review (good!) and grew a new finding — read the rule ID, decide fix vs
acknowledge-with-reason, and remember the house rule: real IDs from real
output only, never invented ones. (2) *The IAM4 ack stops matching*: the
granular id embeds the policy ARN — if you renamed constructs, the
finding's resource path changed; re-run a discovery synth (comment out
the acks) and copy the id it *actually* reports. (3) *Bundling dies*:
cargo-lambda isn't on PATH for the shell `cdk` spawned (IDE terminals
differ from login shells), or the workspace-vs-crate quirk wants
`binaryName`. The failure text names which. And if you want to see the
raw findings the acks are hiding, a discovery synth with the
acknowledgments commented out reproduces NOTES §3's table — three
findings, no more.

</details>

## If truly stuck

Read, don't copy — the stack is Claude's, so here "stuck" means the
*review* stalled, not the code:

- `specs/006-hello-lambda/_reference/NOTES.md` §3 — the full synth
  transcript summary: toolchain versions, the discovery-run findings
  table, the cdk-nag v3 API notes, and the `::` rough edge write-up.
- `infra/README.md` — the auth decision in one breath, the two synth
  modes, and the housekeeping rules (`node_modules/`, `cdk.out/` never
  commit).
- `research/typescript-cdk-for-goldeneye.md` — why Validations-not-
  Aspects, why tags-over-names, and the stateful/stateless split the
  stub stack encodes.
- `specs/006-hello-lambda/design.md` — the Key-decisions table this
  review is checking the implementation against (HTTP layer, auth,
  who-writes-TS).

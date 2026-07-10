# goldeneye infra — CDK v2 TypeScript app

Born in **spec 006-hello-lambda** (H8). Two stacks:

| Stack | File | Status |
|---|---|---|
| `goldeneye-stateless` | `lib/stateless-stack.ts` | live — hello-lambda (RustFunction, ARM64) + Function URL |
| `goldeneye-stateful` | `lib/stateful-stack.ts` | **documented stub** — spec 007 fills it (lake buckets); not instantiated in `bin/goldeneye.ts` |

Region: **us-east-1** (project primary), account-agnostic — deploys into
whatever account your credentials name. Every resource carries the
`project=goldeneye` tag (tags over physical names, per CLAUDE.md).

## Prerequisites

- Node 20+, `npm install` in this directory.
- [cargo-lambda](https://www.cargo-lambda.info/) on `PATH` (plus Zig or the
  `ziglang` Python package for cross-compiling) — `RustFunction` bundles
  locally with it; without it, bundling falls back to Docker
  (`forcedDockerBundling` not required either way).
- A bootstrapped CDK environment for deploys (`npx cdk bootstrap`, once per
  account/region). Synth does not need credentials.

## Synth — two context modes

The stateless stack builds the crate named by the `helloLambdaDir` context
key (path relative to this directory):

```console
# 1. Default: the learner's workspace crate (crates/hello-lambda).
#    Fails with "'…/crates/hello-lambda/Cargo.toml' is not a path to a
#    Cargo.toml file" until that crate exists - expected before sitting O.
$ npx cdk synth

# 2. Reference mode: the validated spec-006 reference implementation
#    (used for authoring/CI validation of the infra itself).
$ npx cdk synth -c helloLambdaDir=../specs/006-hello-lambda/_reference/hello-lambda
```

npm shortcuts: `npm run synth` / `npm run synth:reference`.

## Deploy / destroy (sitting Q — same-sitting teardown is part of the spec)

```console
$ npx cdk deploy goldeneye-stateless        # prints FunctionUrl output
$ npx cdk destroy goldeneye-stateless       # H11: torn down the SAME sitting
```

## The auth decision (Function URL, AuthType=NONE)

The Function URL is deliberately **unauthenticated**. The decision is
gated design (design.md 006, "Function URL auth") and its full written
justification lives as the block comment above the URL in
`lib/stateless-stack.ts`. The bounds, in one breath: single-sitting
lifetime (deployed and destroyed in sitting Q), logs-only blast radius (the
role can write CloudWatch Logs and nothing else; the binary ships no
aws-sdk), worst case an anonymous 400 or a stray JSON line in a log group
that dies with the stack. The IAM alternative is documented next to it for
anyone keeping the endpoint alive longer.

## cdk-nag

Both rule packs CLAUDE.md requires run at synth: **AwsSolutions** and
**Serverless** (cdk-nag **v3**, registered through CDK-native
`Validations.of(app).addPlugins(...)` — not the stale v2 Aspects API).
Synthesis fails on any unacknowledged error-level finding.

Findings and their written acknowledgments (all in `lib/stateless-stack.ts`):

| Rule | Level | Disposition |
|---|---|---|
| `AwsSolutions-IAM4[Policy::…AWSLambdaBasicExecutionRole]` | Error | acknowledged — the managed policy is exactly the logs-only access the function needs |
| `Serverless-LambdaDLQ` | Error | acknowledged — synchronous Function-URL invocation only; a DLQ would never receive a message |
| `Serverless-LambdaTracing` | Warning | acknowledged — H10 reads REPORT lines, not X-Ray traces |
| Function-URL auth | — | **no rule exists** in either pack (cdk-nag 3.0.1); the auth decision is documented in code + spec instead — nothing to suppress |

One rough edge, documented in `specs/006-hello-lambda/_reference/NOTES.md`:
the granular `AwsSolutions-IAM4[Policy::…]` id contains `::`, which
`Validations.acknowledge()` rejects; that single acknowledgment is recorded
via the same public metadata key (`Validations.ACKNOWLEDGED_RULES_METADATA_KEY`)
that `acknowledge()` writes and cdk-nag reads.

## Housekeeping

`node_modules/` and `cdk.out/` are git-ignored (see `.gitignore`) — never
commit them. Keep costs at zero: nothing in this app runs unless deployed,
and the only deployable stack is torn down the sitting it goes up.

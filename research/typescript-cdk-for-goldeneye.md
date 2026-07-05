# TypeScript CDK for goldeneye

**Researched:** 2026-07-05 · **Method:** deep-research workflow (5 angles, 22
sources fetched, 65 claims extracted, 25 verified by 3-vote adversarial panels —
25 confirmed, 0 refuted) · **Decision context:** IaC = CDK v2 + TypeScript
(learner decision, 2026-07-05)

## Executive summary

CDK v2 with TypeScript is a sound, actively-invested choice for goldeneye. AWS's
own Rust-on-Lambda GA announcement (Nov 2025) demonstrates deploying Rust from a
TypeScript CDK app via the `cargo-lambda-cdk` `RustFunction` construct, and
first-party CDK patterns exist for our other targets (Fargate via
`DockerImageAsset`, EC2 via S3 assets + systemd init). One real gap: **Lambda
MicroVMs CloudFormation/CDK support could not be verified to exist** — treat as
missing until proven otherwise. One real tension: AWS best practice says prefer
CDK-generated resource names, which collides with our `goldeneye-` physical-name
convention (resolution below).

## Verified findings

### State of CDK (adopt with confidence)

- **CDK v2 is under active first-party investment**: the official roadmap
  prioritizes expanding L2 constructs and developer experience; 150+ PRs shipped
  Dec 2025–Feb 2026, Mixins stable, EKS v2 L2 GA. Only CDK v1 is in maintenance.
  *(3-0, [roadmap](https://github.com/aws/aws-cdk/blob/main/ROADMAP.md))*
- **Rust on Lambda is GA since Nov 2025** (AWS Support + availability SLA), and
  AWS's *recommended* build tool is Cargo Lambda — third-party open source; even
  SAM CLI delegates Rust builds to it. Rust deploys as a custom-runtime binary on
  `provided.al2023`, not a managed runtime. *(3-0, [AWS blog](https://aws.amazon.com/blogs/compute/building-serverless-applications-with-rust-on-aws-lambda/), [Lambda dev guide](https://docs.aws.amazon.com/lambda/latest/dg/rust-package.html), [InfoQ](https://www.infoq.com/news/2025/11/aws-lambda-rust-support-ga/))*

### Lambda: the construct choice

- **Use `cargo-lambda-cdk`'s `RustFunction`** — maintained by the Cargo Lambda
  project, shown in AWS's own GA blog post. Defaults to `provided.al2023`
  (matches our convention). **ARM64 must be set explicitly** —
  `architecture: Architecture.ARM_64` — the default is x86_64, and mismatched
  bundling/function architectures throw. Bundling: local `cargo-lambda ≥ 0.12.0`
  (no Docker needed, works in CI) or `forcedDockerBundling: true` for
  reproducible builds. Caveat: AWS-endorsed but third-party and pre-1.0
  (v0.0.36, Dec 2025). *(3-0 ×4, [repo](https://github.com/cargo-lambda/cargo-lambda-cdk))*
- **Avoid `cdklabs/aws-lambda-rust`** for now: explicitly experimental (v0.0.10,
  breaking changes allowed), defaults to x86-64. No Rust construct is on the
  official CDK roadmap (Go and Python are; Rust is absent). Watch-later.
  *(3-0 ×4; the x86-default claim passed 2-1 — qualifier: the rustup
  `aarch64-unknown-linux-gnu` target is only needed for local bundling paths)*
- **Do not use `aws-samples/aws-cdk-with-rust`** as a reference: archived
  July 2022, predates RustFunction, covers only Lambda+API GW+DynamoDB. *(3-0)*

### Fargate

- **`DockerImageAsset` builds our multi-stage Rust Dockerfile** and publishes to
  the bootstrap-managed ECR repo — no hand-managed ECR, no custom repo names
  (`repositoryName` was removed), environment must be bootstrapped.
- **Critical ARM64 pitfall (memorize this one):** set
  `platform: Platform.LINUX_ARM64` explicitly (requires Docker Buildx). If
  unset, the image silently builds for the *build machine's* architecture — an
  x86 CI runner produces x86 images that die with exec-format errors on ARM64
  Fargate at task runtime. Modern Docker satisfies Buildx by default; minimal CI
  installs may not. *(3-0 ×2, [DockerImageAsset docs](https://docs.aws.amazon.com/cdk/api/v2/docs/aws-cdk-lib.aws_ecr_assets.DockerImageAsset.html), aws-cdk#12472, #28517)*

### EC2

All three building blocks are first-party documented patterns *(3-0 ×3,
[aws-ec2 README](https://docs.aws.amazon.com/cdk/api/v2/docs/aws-cdk-lib.aws_ec2-readme.html))*:

1. Graviton: `InstanceType.of(InstanceClass.C7G, …)` +
   `MachineImage.latestAmazonLinux2023({ cpuType: AmazonLinuxCpuType.ARM_64 })`.
2. Ship the Rust binary as an S3 asset: `userData.addS3DownloadCommand` +
   `addExecuteFileCommand` + `asset.grantRead(instance.role)`.
3. Long-running daemon: `InitService.systemdConfigFile()` +
   `InitService.enable(…, ServiceManager.SYSTEMD)` — **`InitCommand` cannot
   start long-running processes** (cfn-init waits for exit; deployment times
   out). CDK provisions the ARM64 instance only; cross-compiling the binary for
   aarch64 stays our job.

### Repo layout, stacks, naming (from AWS's official best-practices guide)

- **Infra and runtime code in one repo/package is AWS best practice** —
  validates our plan: `infra/` CDK app beside the cargo workspace. *(3-0)*
- **Split stateful from stateless stacks**: data-lake buckets in their own
  termination-protected stack; compute stacks freely destroyable. (Guide says
  "consider"; credible community dissent exists on its merit, not existence.)
- **Naming tension (real):** hardcoded physical names prevent deploying twice
  per account and block replacement-requiring changes. **Resolution for
  goldeneye:** keep the `goldeneye-` identity via *stack names and tags*
  (`project=goldeneye` on everything); allow hardcoded physical names **only**
  for the two lake buckets (`goldeneye-lake`, `goldeneye-discovery` — they're a
  cross-spec contract); every other resource gets CDK-generated names. *(3-0,
  [best-practices guide](https://docs.aws.amazon.com/cdk/v2/guide/best-practices.html))*

### Guardrails

- **`cdk-nag` at synth time**: use the AwsSolutions + Serverless rule packs (the
  four compliance packs are irrelevant for a learning account). Actively
  maintained (v3.0.1, June 2026) — **v3 registers via CDK-native
  `Validations.of()`, not Aspects**; v2-era snippets online are stale. *(3-0 ×2,
  [repo](https://github.com/cdklabs/cdk-nag))*

## Gaps & open questions (flagged, not guessed)

- **Lambda MicroVMs CDK/CloudFormation support: unverified/unknown.** No claim
  about it survived sourcing. Assume absent at launch; fallback order when we
  get there: `CfnResource` with a raw type if CFN support exists →
  `AwsCustomResource` (SDK calls) → provision outside CDK via CLI/SDK in the
  spec's scripts. Must be resolved inside the future MicroVMs spec.
- Whether cargo-lambda-cdk reaches 1.0 or AWS ships a first-party Rust
  construct — revisit at each AWS-target spec.
- Concrete cost-control wiring (AWS Budgets via CDK; `RemovalPolicy.DESTROY` +
  `autoDeleteObjects` on learning buckets vs the termination-protected stateful
  stack) — to be designed in the first deploying spec.

## Adopt now / later / never

| When | What |
|---|---|
| **Now** (first AWS spec) | `infra/` TypeScript CDK app in-repo; `cargo-lambda-cdk` RustFunction with explicit `ARM_64`; cdk-nag (AwsSolutions + Serverless, v3 API); stateful/stateless stack split; tags-over-names convention |
| **Later** | CDK Pipelines (single learner = plain `cdk deploy` first); Athena/Glue constructs (Wave 2+); budgets stack |
| **Never / avoid** | `cdklabs/aws-lambda-rust` (experimental), `aws-samples/aws-cdk-with-rust` (archived 2022), hand-managed ECR repos, physical names outside the two lake buckets |

## Consequences for goldeneye docs

1. `steering/tech.md`: add construct choices + the two ARM64 explicit-flag rules
   (RustFunction `ARM_64`, DockerImageAsset `LINUX_ARM64`) — both are silent-
   failure traps. ✅ applied 2026-07-05
2. `steering/structure.md`: reserve `infra/` in the repo tree. ✅ applied
3. Naming convention amendment (tags-over-physical-names, bucket exception).
   ✅ applied to tech.md
4. Future MicroVMs spec must carry an explicit "CDK support unknown" risk item.

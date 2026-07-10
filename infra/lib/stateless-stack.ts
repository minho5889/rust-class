/**
 * goldeneye-stateless - the compute stack (spec 006, H8).
 *
 * One Rust Lambda (the 005 door as a handler) behind a Function URL.
 * Freely destroyable by design: deployed and torn down in the same sitting
 * (H11); it holds no state the project would miss.
 */
import * as path from 'node:path';
import { CfnOutput, Duration, Stack, StackProps, Validations } from 'aws-cdk-lib';
import { PolicyStatement } from 'aws-cdk-lib/aws-iam';
import { Architecture, FunctionUrlAuthType } from 'aws-cdk-lib/aws-lambda';
import { RustFunction } from 'cargo-lambda-cdk';
import { Construct } from 'constructs';

export class StatelessStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    // Which crate to build: the learner's workspace crate by default; the
    // validated spec-006 reference when authoring/CI passes
    //   -c helloLambdaDir=../specs/006-hello-lambda/_reference/hello-lambda
    // Paths are relative to infra/ (the cdk.json directory - the CDK CLI
    // always runs the app with cwd there).
    const helloLambdaDir: string =
      this.node.tryGetContext('helloLambdaDir') ?? '../crates/hello-lambda';

    const handler = new RustFunction(this, 'HelloLambda', {
      manifestPath: path.resolve(process.cwd(), helloLambdaDir, 'Cargo.toml'),
      // Needed because the reference crate's manifest carries an empty
      // `[workspace]` table (its opt-out from the repo root workspace);
      // cargo-lambda-cdk reads any [workspace] key as "this is a workspace,
      // name the binary". Harmless for the learner's crate too - its
      // binary has the same name.
      binaryName: 'hello-lambda',
      // ARM64/Graviton EXPLICITLY (CLAUDE.md): cargo-lambda-cdk defaults to
      // x86_64, and an architecture/bundling mismatch is a silent runtime
      // failure. This single line is why the artifact is the same
      // aarch64 bootstrap `cargo lambda build --release --arm64` produces.
      architecture: Architecture.ARM_64,
      // runtime defaults to provided.al2023: no managed runtime, just a
      // Linux that execs our `bootstrap` binary (T2).
      memorySize: 128, // the H10 cold-start experiment's lower rung
      timeout: Duration.seconds(10),
      environment: {
        // Spec 007 (S10): the ingest sink. The bucket name is one of the
        // project's two hardcoded physical names (a cross-spec contract),
        // so the compute stack NAMES it rather than importing the bucket
        // construct - no cross-stack reference means tearing this stack
        // down can never entangle CloudFormation with lake state.
        LAKE_BUCKET: 'goldeneye-lake',
      },
    });

    // Spec 007 (S10): the ingest permission, HAND-WRITTEN on purpose.
    // `lake.grantPut(handler)` would also bundle s3:PutObjectLegalHold,
    // s3:PutObjectRetention, s3:PutObjectTagging, s3:PutObjectVersionTagging
    // and s3:Abort* - none of which this function uses, and whose presence
    // would falsify the "nothing wider than PutObject on raw/*" claim the
    // requirement makes. Least privilege you can literally READ in the
    // synthesized template: one action, one prefix.
    //   - s3:PutObject only: the ingest never lists, never reads, never
    //     deletes (lake-sync's LIST runs as the OPERATOR's credentials,
    //     not the function's).
    //   - resource is the raw/* PREFIX of the one bucket, not the bucket:
    //     S1's "nothing is ever written outside raw/" holds at the IAM
    //     layer too, not just in code.
    handler.addToRolePolicy(
      new PolicyStatement({
        sid: 'GoldeneyeLakeRawPutOnly',
        actions: ['s3:PutObject'],
        resources: ['arn:aws:s3:::goldeneye-lake/raw/*'],
      }),
    );

    // cdk-nag on that statement: AwsSolutions-IAM5 flags ANY wildcard, and
    // the 007 discovery synth reported the granular finding
    //   AwsSolutions-IAM5[Resource::arn:aws:s3:::goldeneye-lake/raw/*]
    // (raw findings list: specs/007-lake-to-s3/_reference/NOTES.md). Here
    // the wildcard IS the least privilege: object keys under raw/ are
    // minted per event (evt-<event_id>.json) and per synced file, so no
    // finite resource list can exist - the prefix is the narrowest
    // expressible grant, and the action list is exactly one action.
    // Same rough edge as 006's IAM4 note: the granular id embeds the
    // ARN's '::' pairs and aws-cdk-lib's Validations.acknowledge() rejects
    // ids with more than one '::' (qualifyId reserves the delimiter), so
    // this acknowledgment travels via the same public metadata key
    // acknowledge() itself writes and cdk-nag reads.
    handler.node.addMetadata(Validations.ACKNOWLEDGED_RULES_METADATA_KEY, {
      'AwsSolutions-IAM5[Resource::arn:aws:s3:::goldeneye-lake/raw/*]':
        'The wildcard is a single PREFIX (goldeneye-lake/raw/*) under one named ' +
        'bucket, paired with exactly one action (s3:PutObject). Ingest keys are ' +
        'minted per event at request time, so a non-wildcard resource list is ' +
        'impossible by construction; the prefix bound enforces S1\'s "nothing is ' +
        'ever written outside raw/" at the IAM layer. No List/Get/Delete/Abort or ' +
        'tagging/retention actions ride along (grantPut was rejected for exactly ' +
        'that reason - see the block comment above).',
    });

    // The lab door: a Function URL with NO auth. This is a deliberate,
    // WRITTEN decision (design.md, Key decisions): see the acknowledge()
    // call below for the full justification and its bounds.
    const url = handler.addFunctionUrl({
      authType: FunctionUrlAuthType.NONE,
    });

    new CfnOutput(this, 'FunctionUrl', {
      value: url.url,
      description: 'POST /events + GET /healthz live here (H9 transcripts)',
    });
    new CfnOutput(this, 'FunctionName', {
      value: handler.functionName,
      description: 'For fetching CloudWatch REPORT lines during H10',
    });

    // ------------------------------------------------------------------
    // cdk-nag acknowledgments (v3 Validations API), each with its written
    // reason - the reasons are the security review, not paperwork.
    //
    // The rule IDs below are the ones a discovery synth (acknowledgments
    // disabled) ACTUALLY reported against this stack - never guessed; the
    // raw findings list lives in specs/006-hello-lambda/_reference/NOTES.md.
    //
    // NOTE the finding neither pack raised: Function URL AuthType=NONE.
    // cdk-nag 3.0.1 (AwsSolutions + Serverless) has no Function-URL auth
    // rule, so the auth decision below carries NO suppression - there is
    // nothing to suppress. The written justification stays (design.md
    // demands the decision in writing), the rule absence is documented in
    // NOTES.md, and if a future cdk-nag adds such a rule, synth will fail
    // and force this text into a real acknowledge() - exactly as it should.
    // ------------------------------------------------------------------

    // AwsSolutions-IAM4 is a GRANULAR rule: the violation id embeds the
    // offending policy ARN, '::' and all - and aws-cdk-lib 2.261's
    // Validations.acknowledge() rejects any id with more than one '::'
    // (qualifyId reserves the delimiter), even though the synth output
    // suggests acknowledging exactly that string. Until that rough edge is
    // fixed upstream, we record this ONE acknowledgment through the same
    // documented metadata key acknowledge() itself writes and cdk-nag
    // reads (Validations.ACKNOWLEDGED_RULES_METADATA_KEY - public API).
    // Full write-up: specs/006-hello-lambda/_reference/NOTES.md.
    handler.node.addMetadata(Validations.ACKNOWLEDGED_RULES_METADATA_KEY, {
      'AwsSolutions-IAM4[Policy::arn:<AWS::Partition>:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole]':
        'The function role carries only the AWS-managed AWSLambdaBasicExecutionRole ' +
        'policy, which grants exactly the CloudWatch Logs write access this lab ' +
        'function needs - stdout/stderr ARE its only outputs (accepted events go to ' +
        'its own log group; no other AWS resource is touched, deliberately: 006 ships ' +
        'no aws-sdk). A customer-managed replacement would duplicate the same three ' +
        'log actions with no scope reduction.',
    });

    Validations.of(handler).acknowledge(
      {
        id: 'Serverless-LambdaDLQ',
        reason:
          'No dead-letter queue: the function is invoked synchronously through its ' +
          'Function URL (request/response), never async or event-sourced, so a DLQ ' +
          'would never receive a message. Failures surface to the caller as HTTP errors.',
      },
      {
        id: 'Serverless-LambdaTracing',
        reason:
          'No X-Ray tracing (warning-level finding): the H10 experiment reads cold-start ' +
          'and duration numbers straight from CloudWatch REPORT lines, which exist ' +
          'without X-Ray; active tracing would add a sidecar cost and an SDK surface to ' +
          'a single-sitting lab function measured for its minimal footprint.',
      },
    );

    // The auth decision, in writing (design.md "Function URL auth"):
    // AuthType=NONE is acceptable ONLY under all of these bounds -
    //   1. lifetime: deployed and destroyed in the same sitting (H11); it
    //      is never left running unattended;
    //   2. blast radius: the function writes only to its own CloudWatch log
    //      group; it can reach no data store, no lake, no other resource
    //      (the role is logs-only, and 006 deliberately ships no aws-sdk);
    //   3. exposure: worst case is an internet stranger paying us a 400, or
    //      donating a syntactically-valid JSON line to a log group that
    //      dies with the stack;
    //   4. the alternative (AWS_IAM + SigV4-signed curls) would consume the
    //      sitting on request signing instead of the actual lesson.
    // No cdk-nag acknowledge() accompanies this decision because neither
    // installed rule pack has a Function-URL-auth rule to acknowledge (see
    // the block comment above and NOTES.md).
    // The IAM variant for anyone keeping the endpoint alive longer:
    //   handler.addFunctionUrl({ authType: FunctionUrlAuthType.AWS_IAM })
    //   and sign requests with `curl --aws-sigv4 "aws:amz:us-east-1:lambda"`.
  }
}

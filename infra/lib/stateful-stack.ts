/**
 * goldeneye-stateful - the lake's home (spec 007, S10).
 *
 * The two S3 buckets that ARE the project's state: `goldeneye-lake` (raw
 * telemetry zones; lake-sync's target; the ingest Lambda's sink) and
 * `goldeneye-discovery` (the research/insight zone - created NOW so the
 * lake's stateful footprint is born complete, but its purpose is Wave 3
 * per datalake/README.md; no Phase-1 requirement writes to it). These are
 * the ONLY two resources in the project allowed hardcoded physical names
 * (a cross-spec contract; everything else is tags-over-names, CLAUDE.md).
 *
 * Everything here is chosen to make the stack UN-lose-able and the specs'
 * laws true:
 *
 *  - `terminationProtection: true` sits on the STACK (set at the
 *    bin/goldeneye.ts instantiation): `cdk destroy` refuses until a human
 *    flips protection off in two deliberate steps.
 *  - `RemovalPolicy.RETAIN` sits on each BUCKET: even if the stack IS
 *    destroyed, CloudFormation orphans the buckets instead of emptying
 *    the lake. Two independent safeties, two different failure modes.
 *  - SSE-S3 (`S3_MANAGED`), NOT SSE-KMS - deliberately: encryption at
 *    rest with zero key management, and single-part PUT etags stay
 *    body-md5, which is the premise lake-sync's idempotence compare
 *    stands on (design.md etag fact #2; the honest paragraph lives in
 *    lake-store's s3.rs). Switching to KMS would silently turn
 *    `uploaded 0` into `uploaded everything, every run`.
 *  - Versioning OFF (S10, explicit): the lake is append-shaped (per-file
 *    sync + per-event ingest, no deletes in Phase 1); version noise would
 *    only complicate the conservation story and add cents.
 *  - All public access blocked; SSL-only transport enforced via the
 *    bucket policy `enforceSSL` writes (aws:SecureTransport deny).
 *
 * Split from the stateless stack ON PURPOSE (006 committed to this): the
 * compute stack deploys and tears down in one sitting; this stack, once
 * deployed, is never torn down casually - it is the lake.
 */
import { RemovalPolicy, Stack, StackProps, Validations } from 'aws-cdk-lib';
import { BlockPublicAccess, Bucket, BucketEncryption } from 'aws-cdk-lib/aws-s3';
import { Construct } from 'constructs';

export class StatefulStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    // One helper so the two buckets CANNOT drift apart in posture; the
    // name is the only difference between them.
    const lakeBucket = (constructId: string, bucketName: string): Bucket =>
      new Bucket(this, constructId, {
        bucketName,
        encryption: BucketEncryption.S3_MANAGED, // SSE-S3: see header - etag=md5 premise
        blockPublicAccess: BlockPublicAccess.BLOCK_ALL,
        enforceSSL: true, // deny non-TLS requests via bucket policy
        versioned: false, // S10: versioning off, explicitly
        removalPolicy: RemovalPolicy.RETAIN, // bucket outlives even a stack delete
      });

    const lake = lakeBucket('LakeBucket', 'goldeneye-lake');
    const discovery = lakeBucket('DiscoveryBucket', 'goldeneye-discovery');

    // ------------------------------------------------------------------
    // cdk-nag acknowledgments (v3 Validations API), each with its written
    // reason. The rule ID below is what a discovery synth (acknowledgments
    // disabled) ACTUALLY reported against this stack - never guessed; the
    // raw findings list lives in specs/007-lake-to-s3/_reference/NOTES.md.
    // ------------------------------------------------------------------
    Validations.of(lake).acknowledge({
      id: 'AwsSolutions-S1',
      reason:
        'No server access logging: a single-operator lab lake written by two ' +
        'first-party producers (lake-sync from one machine, one ingest Lambda ' +
        'whose invocations already log to CloudWatch). Access logs would need ' +
        'a THIRD bucket that itself flags S1, roughly doubling the stateful ' +
        'footprint to audit cents-per-month of telemetry no third party can ' +
        'reach (BPA on, SSL enforced, no public policy). Revisit if the lake ' +
        'ever holds data with an audience.',
    });
    Validations.of(discovery).acknowledge({
      id: 'AwsSolutions-S1',
      reason:
        'Same posture as goldeneye-lake (see its acknowledgment): the ' +
        'discovery bucket is born empty for Wave 3 (datalake/README.md) and ' +
        'has no Phase-1 writers at all; an access-log bucket for an empty ' +
        'bucket would be pure ceremony.',
    });
  }
}

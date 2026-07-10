/**
 * goldeneye-stateful - DOCUMENTED STUB, filled by spec 007.
 *
 * This stack will own the data-lake storage: the `goldeneye-lake` and
 * `goldeneye-discovery` buckets - the ONLY two resources in the project
 * allowed hardcoded physical names (they are a cross-spec contract;
 * everything else is tags-over-names, per CLAUDE.md and
 * research/typescript-cdk-for-goldeneye.md).
 *
 * Why it exists now, empty (H8): the stateful/stateless split is a
 * structural decision 006 already commits to - compute stacks are freely
 * destroyable (deploy and tear down in one sitting), while lake state, once
 * it exists, will be termination-protected and never torn down casually.
 * Having the file in place makes the split visible at review time and gives
 * 007 a named home instead of a refactor.
 *
 * NOT instantiated in bin/goldeneye.ts - 007 adds the `new StatefulStack`
 * call when there is actually state to hold. Expected shape when it lands:
 *   - two S3 buckets (versioned, SSL-enforced, access-logged or
 *     nag-suppressed with reasons), hardcoded names as above;
 *   - `terminationProtection: true` on the stack;
 *   - RemovalPolicy decisions made explicitly, in writing, at the 007 gate.
 */
import { Stack, StackProps } from 'aws-cdk-lib';
import { Construct } from 'constructs';

export class StatefulStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);
    // Intentionally empty until spec 007 (the lake's S3 home).
  }
}

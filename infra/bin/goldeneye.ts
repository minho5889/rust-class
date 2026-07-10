#!/usr/bin/env node
/**
 * goldeneye CDK app (born in spec 006-hello-lambda).
 *
 * App-wide wiring lives here and ONLY here:
 *  - the `project=goldeneye` tag on everything (CLAUDE.md: tags over
 *    physical names),
 *  - cdk-nag registration - v3 registers through CDK-native
 *    `Validations.of().addPlugins()` (the Aspects-based snippets online are
 *    the stale v2 API; see research/typescript-cdk-for-goldeneye.md), with
 *    BOTH rule packs CLAUDE.md requires: AwsSolutions + Serverless,
 *  - stack instantiation, pinned to us-east-1 (the primary region) but
 *    account-agnostic: whatever account the deployer's credentials name.
 */
import * as cdk from 'aws-cdk-lib';
import { AwsSolutionsChecks, ServerlessChecks } from 'cdk-nag';
import { StatefulStack } from '../lib/stateful-stack';
import { StatelessStack } from '../lib/stateless-stack';

const app = new cdk.App();

// Every taggable resource in every goldeneye stack carries the project tag.
cdk.Tags.of(app).add('project', 'goldeneye');

// cdk-nag v3: synthesis fails on any unacknowledged error-level finding.
// Suppressions are per-construct `Validations.of().acknowledge()` calls in
// the stacks, each carrying a written justification - the reason strings
// are part of the review surface, not boilerplate.
cdk.Validations.of(app).addPlugins(
  new AwsSolutionsChecks(app, { verbose: true }),
  new ServerlessChecks(app, { verbose: true }),
);

new StatelessStack(app, 'goldeneye-stateless', {
  env: { region: 'us-east-1' },
  description:
    'goldeneye stateless compute: hello-lambda (specs 006/007) - freely destroyable, torn down the same sitting it is deployed',
});

// The STATEFUL stack, instantiated by spec 007 exactly as 006 promised.
// terminationProtection lives HERE, on the stack (S10): `cdk destroy
// goldeneye-stateful` refuses until a human disables protection first -
// the buckets' own RemovalPolicy.RETAIN (in the stack file) is the second,
// independent safety. Deploys of goldeneye-stateless never touch this
// stack; there is no cross-stack reference on purpose (the bucket name is
// a fixed contract, so the compute stack names it instead of importing it
// - tearing compute down can never tangle with lake state).
new StatefulStack(app, 'goldeneye-stateful', {
  env: { region: 'us-east-1' },
  terminationProtection: true,
  description:
    'goldeneye stateful: the goldeneye-lake / goldeneye-discovery buckets (spec 007) - termination-protected, RETAIN, never torn down casually: it IS the lake',
});

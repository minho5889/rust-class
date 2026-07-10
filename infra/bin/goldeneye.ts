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
    'goldeneye stateless compute: hello-lambda (spec 006) - freely destroyable, torn down the same sitting it is deployed',
});

// The STATEFUL stack (goldeneye-lake / goldeneye-discovery buckets,
// termination-protected) is deliberately NOT instantiated yet: spec 007
// fills lib/stateful-stack.ts and adds the `new StatefulStack(...)` call
// here. Keeping the file present-but-uninstantiated means 006's deploys and
// teardowns can never touch lake state by accident.

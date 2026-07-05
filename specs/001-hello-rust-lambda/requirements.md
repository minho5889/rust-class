# Requirements — 001 Hello Rust Lambda

> **Phase: Inception (Mob Elaboration).** Drafted by Claude as the first Unit of
> Work. Awaiting learner approval before design begins.

**Status:** draft

## Intent

Ship the learner's first Rust code to AWS: a minimal Lambda function built with
`cargo-lambda`, deployed on ARM64, invoked with a JSON payload. The purpose is not
the function itself but the end-to-end loop — write Rust, cross-compile, deploy,
invoke, read CloudWatch logs — while exercising SKILLS.md Level 0 (toolchain) and
Level 1a/1b basics (ownership on `String` payloads, `Result`, `serde` structs).

## User stories

- As a learner, I want to deploy a working Rust Lambda within one session, so that
  I have a fast feedback loop for everything that follows.
- As a learner, I want to see the actual cold-start and memory numbers of my
  function, so that Rust's efficiency claims become concrete.

## Acceptance criteria (EARS notation)

1. WHEN the function is invoked with `{"name": "<string>"}` THE SYSTEM SHALL return
   `{"message": "Hello, <string>!"}` with a 200-equivalent success response.
2. IF the payload is missing the `name` field THEN THE SYSTEM SHALL return a
   structured error (not a panic) that is visible in CloudWatch logs.
3. THE SYSTEM SHALL be built for `arm64` and run on the `provided.al2023` runtime.
4. WHEN the function cold-starts THE SYSTEM SHALL log its init duration, and the
   learner SHALL record the observed cold-start time in this spec's tasks.md.

## Learning requirements

1. The learner SHALL be able to explain why the handler takes ownership of (or
   borrows) the event payload, and what `serde` deserialization allocates.
2. The learner SHALL be able to explain what `Result<T, E>` and the `?` operator do
   in the handler.
3. The learner SHALL be able to rebuild and redeploy the function unaided.

## Out of scope

- API Gateway / HTTP routing (later unit).
- Infrastructure as code (Phase 2 decision).
- MicroVMs, Fargate, EC2 (units 002+).

## Open questions (answered before approval)

- [ ] Which AWS region? (Recommend one that also supports Lambda MicroVMs so later
      units stay in the same region — e.g., `us-east-1` or `ap-northeast-1`.)
- [ ] Is an AWS account with credentials ready on the learner's machine?

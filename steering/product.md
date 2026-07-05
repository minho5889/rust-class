# Steering — Product

## Why this project exists

`rust-class` is a personal learning environment, not a product for end users. Its
"customer" is one learner (Minho) whose goal is to become productive in **Rust for
systems engineering on AWS**, exploiting Rust's memory-management model for
efficiency (low cold-start latency, small memory footprint, low cost).

## Success criteria

1. The learner can independently write, test, and deploy idiomatic Rust to all four
   AWS compute targets: Lambda, Lambda MicroVMs, ECS Fargate, EC2.
2. The learner can explain *why* Rust is efficient on each target — ownership,
   no GC pauses, small binaries — not just follow recipes.
3. Every deployed artifact has a spec trail (`specs/NNN-*/`) showing the
   AI-DLC path from intent → requirements → design → tasks → operation.

## Non-goals

- Production traffic or real users.
- Exhaustive AWS coverage — only the four compute targets above (plus the minimal
  supporting services they require: IAM, CloudWatch, ECR, API Gateway/ALB).
- Rust web-frontend/WASM topics (out of scope for now; revisit later if desired).

## Guiding principle

Every exercise should teach a memory-management or efficiency lesson that is
*observable on AWS* — a cold-start number, a RAM ceiling, a cost line item.

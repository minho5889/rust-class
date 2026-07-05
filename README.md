# rust-class 🦀 ☁️

A structured, AI-driven learning environment for **Rust on AWS** — Lambda,
Lambda MicroVMs, ECS Fargate, and EC2 — with a focus on Rust's memory-management
model and why it makes systems efficient (and cheap) in the cloud.

## How it works

This repo runs on the **AI-DLC** methodology (AWS's AI-Driven Development
Lifecycle) combined with **Kiro-style spec-driven development** — borrowed as a
process, no Kiro IDE required. The AI assistant drives; the learner validates
every decision at explicit gates.

Every piece of work is a **Unit of Work** in `specs/NNN-name/` that moves through:

1. **Inception** — `requirements.md` (EARS notation), approved by the learner
2. **Construction** — `design.md` → `tasks.md`, implemented in short **bolts**
3. **Operations** — deploy to the real AWS target, observe, record what was learned

## Start here

| File | What it is |
|---|---|
| [`CLAUDE.md`](CLAUDE.md) | The constitution — how the AI assistant works in this repo |
| [`MEMORY.md`](MEMORY.md) | Living memory — learner profile, progress, decisions, session log |
| [`SKILLS.md`](SKILLS.md) | The curriculum — a skill tree from Rust fundamentals to a four-target cost benchmark |
| [`steering/`](steering/) | Always-on context: product goals, tech stack, repo structure |
| [`specs/`](specs/) | Units of Work — first up: `001-hello-rust-lambda` |

## The four compute targets

| Target | Why it's in the curriculum |
|---|---|
| **AWS Lambda** | Fastest feedback loop; Rust's ~15 ms cold starts make the efficiency story visceral |
| **AWS Lambda MicroVMs** (June 2026) | Firecracker-based stateful, isolated sandboxes — the newest serverless primitive |
| **ECS Fargate** | Containerized long-running services; static Rust binaries in `scratch` images |
| **EC2** | The baseline: a systemd-managed Rust daemon, and the ops burden comparison |

# Steering — Repository Structure

```
rust-class/  (project: goldeneye)
├── CLAUDE.md              # Constitution v2: 4-doc pipeline, gates, PBT, telemetry
├── MEMORY.md              # Living memory: profile, progress, decisions, session log
├── SKILLS.md              # Skill tree / curriculum with mastery tracking
├── README.md              # Human-facing overview
├── steering/              # Kiro-style always-loaded context
│   ├── product.md         #   why (goals, non-goals)
│   ├── tech.md            #   stack, targets, testing, data lake, constraints
│   └── structure.md       #   this file
├── specs/                 # AI-DLC Units of Work (four-doc pipeline)
│   ├── _template/         #   copy to start a new Unit of Work
│   │   ├── intent.md      #     doc 1 — write-once interpretation contract
│   │   ├── requirements.md#     doc 2 — EARS + [P]/[E]/[O] tags   ✋ gate
│   │   ├── design.md      #     doc 3 — shape + Properties table  ✋ gate
│   │   ├── tasks.md       #     doc 4 — main/sub/action layers    ✋ gate
│   │   ├── evidence.md    #     append-only report card (no gate)
│   │   └── _assurance/    #     machine-review sidecar (bots write here only)
│   └── NNN-short-name/    #   numbered sequentially (002, 003, …)
├── datalake/              # goldeneye telemetry
│   ├── schema/            #   envelope.v1.json (schema registry, versioned)
│   ├── raw-local/         #   dt=YYYY-MM-DD/events.jsonl (v1 raw zone, committed)
│   └── README.md          #   zones, S3 naming, conventions
├── .claude/
│   ├── agents/            #   intent-assurance, spec-auditor, property-auditor
│   ├── hooks/             #   telemetry.sh, on-doc-write.sh
│   └── settings.json      #   hook wiring (SessionStart/Stop/PostToolUse)
├── research/              # Verified deep-research reports (<topic>.md, no dates)
├── infra/                 # TypeScript CDK v2 app (created with first AWS spec)
├── playground/            # Throwaway experiments — no spec required
└── crates/                # Cargo workspace members (created as class progresses)
    ├── shared/            #   shared types/utilities
    └── <deployable>/      #   one crate per deployable unit
```

## Conventions

- **Spec directories**: `specs/NNN-kebab-case/`, numbered in creation order starting
  at 002 (001 was retired before pipeline v2; see MEMORY.md). Specs are never
  deleted; superseded specs get a note pointing to the successor.
- **Status lines**: gated docs carry `**Status:** drafting | awaiting-review |
  revising | approved | superseded` — exact format matters, hooks grep it.
- **Crate names**: kebab-case matching their directory. AWS resource names:
  `goldeneye-<purpose>`.
- **Workspace**: root `Cargo.toml` with `members = ["crates/*"]` once the first
  crate exists. Playground exercises stay outside the workspace.
- **Branches & merges** (adopted 2026-07-05, effective from spec 002's close):
  work happens on a branch per spec (`claude/NNN-short-name`; the current
  `claude/rust-aws-learning-44il52` carries everything through 002). When a spec
  closes (all gates passed, evidence written), merge to `main` and tag
  `spec/NNN-short-name` — `main` is the sum of approved+verified work, and tags
  give the data lake clean spec-boundary markers.
- **Commits**: imperative mood, reference the spec
  (`002: implement handler (task 1.1.2)`).
- **Teaching docs**: `///` doc comments in code; learnings in each spec's
  `evidence.md`, promoted to `MEMORY.md`/`SKILLS.md`.

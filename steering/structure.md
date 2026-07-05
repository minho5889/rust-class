# Steering — Repository Structure

```
rust-class/
├── CLAUDE.md              # Constitution: how Claude works here (read first)
├── MEMORY.md              # Living memory: profile, progress, decisions, session log
├── SKILLS.md              # Skill tree / curriculum with mastery tracking
├── README.md              # Human-facing overview
├── steering/              # Kiro-style always-loaded context
│   ├── product.md         #   why (goals, non-goals)
│   ├── tech.md            #   stack, targets, constraints
│   └── structure.md       #   this file
├── specs/                 # AI-DLC Units of Work (spec-driven development)
│   ├── _template/         #   copy to start a new Unit of Work
│   │   ├── requirements.md
│   │   ├── design.md
│   │   └── tasks.md
│   └── NNN-short-name/    #   numbered sequentially (001, 002, …)
├── playground/            # Throwaway experiments — no spec required
│   └── NN-topic/          #   small numbered exercises (e.g., 01-ownership)
└── crates/                # Real Cargo workspace members (created as class progresses)
    ├── shared/            #   shared types/utilities across deployables
    └── <deployable>/      #   one crate per deployable unit (lambda fn, service…)
```

## Conventions

- **Spec directories**: `specs/NNN-kebab-case/`, numbered in creation order. A spec
  is never deleted; superseded specs get a note at the top pointing to the successor.
- **Crate names**: kebab-case matching their directory (`hello-rust-lambda`).
- **Workspace**: root `Cargo.toml` uses `[workspace]` with `members = ["crates/*"]`
  once the first crate exists. Playground exercises are standalone, excluded from
  the workspace.
- **Branches**: work happens on `claude/rust-aws-learning-*` branches.
- **Commits**: imperative mood, reference the spec (`001: implement handler (task 2.1)`).
- **Docs style**: explanations for the learner live as doc comments (`///`) in code
  and as "What you learned" notes appended to each spec after Operations.

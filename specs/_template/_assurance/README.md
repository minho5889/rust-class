# _assurance/ — machine review sidecar

Independent audit output for this spec. Written by fresh-context subagents and by
the counterexample triage protocol. **Humans read this; bots never edit the spec
docs themselves.**

| File | Written by | When |
|---|---|---|
| `intent-review.md` | `intent-assurance` agent | After `intent.md` is created |
| `requirements-review.md` | `spec-auditor` agent | Before requirements go to `awaiting-review` |
| `design-review.md` | `spec-auditor` agent | Before design goes to `awaiting-review` |
| `property-audit.md` | `property-auditor` agent | Before the spec closes |
| `triage-log.md` | main session | Every property counterexample: spec/code/test-bug classification |

Each review ends with a one-line verdict (`VERDICT: <confidence>% — <summary>`)
that gets embedded in the human review notification.

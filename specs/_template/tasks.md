# Tasks — <Unit of Work name>

> **Phase: Construction (Mob Construction, part 2).** Derived from the approved
> design. Tasks are grouped into dependency **waves**: tasks within a wave are
> independent and can be done in a single bolt; waves run in order. Check tasks
> off as they complete; note deviations inline.

## Wave 1

- [ ] 1.1 …
- [ ] 1.2 …

## Wave 2 (depends on Wave 1)

- [ ] 2.1 …

## Operations checklist (after implementation)

- [ ] `cargo fmt` and `cargo clippy -- -D warnings` clean
- [ ] Tests pass locally
- [ ] Deployed to target and verified with a real invocation
- [ ] Observed in CloudWatch (logs/metrics) — record the cold start / memory numbers
- [ ] Torn down (if the resource costs money at idle)
- [ ] "What you learned" note appended below; `MEMORY.md` and `SKILLS.md` updated

## What you learned

_(Filled in at the end of the Operations phase.)_

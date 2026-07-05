# Counterexample triage log — 002 memory-lens

## 2026-07-05 — R2 adversarial suite: duplicated no-op realloc

- **Counterexample (shrunk, seed committed in
  `crates/memlens-replay/tests/prop_engine.proptest-regressions`):** a trace
  whose `Grow` hits the generator's 262,144-byte clamp with an even factor →
  `Realloc { old_addr == new_addr, old_size == new_size }` — a semantic
  no-op. Mutation 1 ("duplicate a heap event ⇒ trace must become invalid")
  duplicated it; the duplicated trace is *genuinely valid*; `validate()`
  correctly accepted it; `prop_assert!(is_err())` failed.
- **Found by:** proptest during the full-workspace run in the operations
  bolt; seed auto-committed without triage (process miss — caught by the
  property-auditor's pre-close audit, verdict 75%, blocking).
- **Classification: TEST BUG.** The mutation strategy's universal claim
  ("any duplication invalidates") is unsound for exact-no-op reallocs. The
  validator's acceptance is correct behavior. No spec amendment needed
  (R2's requirement text is untouched); no code change needed.
- **Fix:** mutation 1 now excludes exact no-op reallocs from duplication
  targets (in-place reallocs with *different* sizes remain targets — their
  duplication is genuinely invalid, old_size mismatch). Seed kept as a
  permanent regression case; suite green.
- **Lesson (for the class, later):** shrinking handed us the minimal
  counterexample — the boundary artifact of our own `clamp()` — and the
  failing claim was in the *test's model of validity*, not the system.
  Property-based testing audits the auditor too.

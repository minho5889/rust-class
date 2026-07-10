# Tasks — 006 hello-lambda

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

No new ramp steps — the ramp completed at 13; everything 006 needs (async,
traits, the door) you already own. **Three sittings** (O→Q): build the
handler locally, build the artifact and the infrastructure, then deploy day.
You write the Rust; Claude wrote the TypeScript (you review it); properties
come red-first; one commit per move. Materials (sitting guides O–Q) are
authored and validated against the 006 reference **before this gate is
presented**.

**Done means:** the same curl transcripts from your laptop against a real
`lambda-url.us-east-1.on.aws` URL, a cold-start table in evidence.md with
your own numbers, and the stack torn down before you stand up.

---

## 0. Ramp

*(none — complete at step 13; sitting O opens with a 10-minute recall drill
instead: the 005 door contract from memory)*

## 1. hello-lambda, three sittings (learner writes Rust; Claude coaches)

### Sitting O — the server you don't write *(H1–H5; T1, T2, T4)*
- [ ] 1.1 **H4 equivalence property first** (red, co-written): 005's message
      generator, hello-lambda's door stubbed (`todo!()`), relay compared via
      `oneshot`. *(commit: red)*
- [ ] 1.2 `cargo new crates/hello-lambda`; `lambda_http` + glake dep; the
      handler: door → 202/stdout-line/400, healthz from `OnceLock` state,
      404 else; H1–H3/H5 tests green; H4 green; lints denied (H7).
      The T4 conversation happens here: what do these counters mean *now*?
      *(commits: handler, tests+green)*

### Sitting P — the artifact and the stack *(H6–H8; T2, T3)*
- [ ] 1.3 The build (you drive): `cargo lambda build --release --arm64`;
      `file` the bootstrap; record size + `cargo bloat` top-10 → evidence
      draft. Compare against the workspace release profile — say which
      flags bought what. *(commit: build notes with the crate)*
- [ ] 1.4 The stack (Claude drove, you review as the AWS professional):
      read `infra/lib/stateless-stack.ts` end-to-end; challenge the
      Function-URL AuthType=NONE suppression text — if the written bound
      doesn't convince you, we change the design before deploying; run
      `npx cdk synth` and read the template diff-style: ARM_64, AL2023,
      tags, nag report. *(commit: any review-driven infra edits)*

### Sitting Q — deploy day *(H9–H11; T5)* *(learner's account; Claude co-drives)*
- [ ] 1.5 `cdk deploy goldeneye-stateless` → curl the three transcripts →
      find your event line in CloudWatch → **the cold-start experiment**
      (H10: ≥5 cold starts × {128, 512} MB, tabulated) → `cdk destroy` same
      sitting → evidence.md gets the numbers, timestamps, and cost note.
      *(commits: evidence)*

## 2. Close-out (Claude, machine work)

- [ ] 2.1 evidence.md finalized (T1–T5 notes, H6 sizes, H10 table, property
      outcome); property-auditor run; SKILLS 2a updates; MEMORY; main FF +
      `spec-close/006-hello-lambda` marker.

---

## Operations checklist

- [ ] fmt + clippy clean (`unwrap_used`/`expect_used` denied); H4 ≥256 cases,
      red-first, seed committed on genuine failure; H1–H3/H5 tests green;
      H6 artifact + size recorded; H8 synth + nag clean-or-suppressed;
      H9–H11 evidence from deploy day; **deploy AND teardown both logged**;
      property-auditor pass.

*Deviation declared: deploy/teardown happen in sitting Q on the learner's
account, not during authoring (no credentials in the authoring session) —
the split is tagged per-requirement in requirements.md.*

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-10 | Initial fast-path draft | Part-5 directive (full loop) | pending combined ack |

</details>

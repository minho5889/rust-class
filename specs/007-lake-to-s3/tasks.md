# Tasks — 007 lake-to-s3

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

No new ramp steps. **Four sittings** (R→U): design the seam and prove the
laws, meet the real SDK and pay its bill, evolve the Lambda and the stacks,
then deploy day two. You write the Rust; Claude wrote the stateful stack TS
(you review it); properties red-first; one commit per move. Materials
(sitting guides R–U) are authored and validated against the 007 reference
**before this gate is presented**.

**Done means:** your real backlog lives in `s3://goldeneye-lake/raw/`,
running sync twice uploads nothing, one curl puts one object in the bucket,
DuckDB answers the standing queries over `s3://` with the same totals as
`glake stats` — and the only thing left standing afterwards is the lake.

---

## 0. Ramp

*(none — complete at 13. Sitting R opens by re-reading your own 004
`EventParser` trait: today you design the second trait of your life.)*

## 1. lake-to-s3, four sittings (learner writes Rust; Claude coaches)

### Sitting R — the seam and the laws *(S1–S4; T1, T2, T5)*
- [ ] 1.1 Design `ObjectStore` + `ObjectMeta` + `StoreError` together
      (co-written rubric pass, S7 groundwork); `cargo new crates/lake-sync`
      (lib+bin); `FakeStore` with the ledger (puts counted, gauge).
      *(commit: the seam)*
- [ ] 1.2 **S3 conservation + S4 idempotence properties first** (red,
      co-written): tempdir lake generator adapted from your R9 generator,
      against a stub `plan`/`run` core. *(commit: red)*
- [ ] 1.3 The pure core: `plan` (walk + list + size/md5 compare) and `run`
      (execute against any `S: ObjectStore`); S1/S2 examples; S3/S4 green.
      *(commits: plan, laws green)*

### Sitting S — reality and the bill *(S5, S8, S9; T3, T4)*
- [ ] 1.4 Streaming + the bound: one reused line buffer; `JoinSet` +
      `Semaphore(4)`; **S9 gauge test** (many-file generated lake never
      exceeds 4 in flight); S5 error paths + CLI examples.
      *(commits: streaming, bound+errors)*
- [ ] 1.5 `S3Store` (aws-sdk-s3, ~60 lines, read together line by line);
      clap wiring; **the bill (S8, you drive):** ARM64 builds of 006
      baseline vs ingest-to-be + lake-sync; sizes and `cargo bloat` deltas
      → evidence draft; Graviton crypto-backend note recorded.
      *(commits: s3store, the-bill notes)*

### Sitting T — the Lambda grows a real sink + the stateful stack *(S6, S10, S11)*
- [ ] 1.6 Evolve `crates/hello-lambda`: `println!` → `store.put(...)`
      (client-once `OnceLock`, `LAKE_BUCKET` env); **S6 fake-store tests**;
      006's H4 property re-run green (the door didn't move).
      *(commits: sink swap, tests green)*
- [ ] 1.7 The stateful stack (Claude drove, you review as the AWS pro):
      buckets, RETAIN + termination protection, the scoped `raw/*` grant —
      challenge anything; `cdk synth` + nag both stacks. **Scan pack**: run
      the local DuckDB variants against your real lake; totals must match
      `glake stats`; read the s3 variants' diff (path + httpfs only).
      *(commits: infra review edits, scan pack outputs)*

### Sitting U — deploy day two *(S12; T5, T6)* *(learner's account; Claude co-drives)*
- [ ] 1.8 Stateful deploy (buckets born, protection verified) → real
      backlog sync (numbers vs dry-run plan; re-run shows `uploaded 0`) →
      ingest deploy → one curl, one object → DuckDB-over-S3 scan pack vs
      `glake stats` → stateless teardown, **buckets stay** → evidence.md
      (numbers, timestamps, cost note). *(commits: evidence)*

## 2. Close-out (Claude, machine work)

- [ ] 2.1 evidence.md finalized (T1–T6, S8 bill, S12 transcripts, property
      outcomes); property-auditor run; SKILLS 2e/2a updates; MEMORY; main FF
      + `spec-close/007-lake-to-s3` marker; datalake/README Wave-2 status
      flipped.

---

## Operations checklist

- [ ] fmt + clippy clean (`unwrap_used`/`expect_used` denied, both crates);
      S3/S4 ≥256 cases red-first (seeds committed on genuine failure);
      S1/S2/S5/S6/S9 examples green; H4 re-run green; S8 bill recorded;
      both ARM64 artifacts built; synth + nag clean-or-suppressed; scan pack
      local runs recorded; S12 deploy-day evidence incl. **buckets retained,
      compute torn down**; property-auditor pass.

*Deviation declared: deploy/teardown happen in sitting U on the learner's
account (no credentials in the authoring session); the stateful stack is
intentionally NOT torn down — it is the lake.*

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-10 | Initial fast-path draft | Part-6 directive (full loop) | pending combined ack |

</details>

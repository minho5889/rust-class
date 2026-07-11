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
      (co-written rubric pass, S7 groundwork) — **including the Send
      moment**: write it as bare `async fn` first, meet the `JoinSet` wall
      (predicted before compiling), then desugar to RPITIT `+ Send`.
      `cargo new crates/lake-store` (the seam crate) and
      `crates/lake-sync` (lib+bin); `FakeStore` behind the `fake` feature
      (put-replaces-key, ledger with the gauge **inside `put`**).
      *(commits: the seam, the fake)*
- [ ] 1.2 **S3 conservation + S4 idempotence properties first** (red,
      co-written): tempdir lake generator adapted from your R9 generator —
      domain includes `traces/` and `dt=bad-ts` arms — against a stub
      `plan`/`run` core. *(commit: red)*
- [ ] 1.3 The pure core: `plan` (walk + list + size/etag compare;
      key = `raw/` + relative path) and `run` (execute against `Arc<S>`);
      S1/S2 examples; S3/S4 green — incl. the same-size mutation arm and
      the conservation re-check after re-sync. *(commits: plan, laws green)*

### Sitting S — reality and the bill *(S5, S8, S9; T3, T4)*
- [ ] 1.4 Streaming + the bound: one reused line buffer; `JoinSet` +
      `Arc<Semaphore>(4)` with `acquire_owned`; **S9 gauge test**
      (many-file generated lake never exceeds 4 in flight, measured inside
      `put`); S5 error paths incl. the fake's `fail_on` mid-run failure +
      CLI examples. *(commits: streaming, bound+errors)*
- [ ] 1.5 `S3Store` (aws-sdk-s3, ~60 lines, read together line by line —
      pause at the etag quote-strip and say why it exists); clap wiring;
      **the bill, part one (S8, you drive):** ARM64 builds of the 006
      baseline + lake-sync; sizes and `cargo bloat` → evidence draft, plus
      a **written prediction** of what the ingest build will weigh (the
      evolved Lambda doesn't exist until sitting T fills the row — the
      prediction-then-measurement is the experiment shape); checksum-crate
      Graviton note recorded. *(commits: s3store, the-bill notes)*

### Sitting T — the Lambda grows a real sink + the stateful stack *(S6, S6b, S10, S11)*
- [ ] 1.6 Evolve `crates/hello-lambda` at the 006 emit seam: the returned
      line goes to `store.put(...)` instead of `println!` (client built
      once in async `main`, parked in `OnceLock`; `LAKE_BUCKET` env);
      **S6 fake-store tests**; 006's H4 property re-run green (the door
      didn't move); **the bill, part two:** the ingest ARM64 build fills
      the S8 row sitting S predicted. **Change protocol (S6b):** amend 006 requirements —
      H1's stdout sink superseded, H7's no-SDK rule superseded for the
      evolved crate — changelog entries in 006, re-gated with this spec's
      close. *(commits: sink swap, tests green, 006 amendment)*
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

- [ ] fmt + clippy clean (`unwrap_used`/`expect_used` denied, all three
      crates); S3/S4 ≥256 cases red-first incl. same-size arm + re-sync
      conservation re-check (seeds committed on genuine failure);
      S1/S2/S5/S6/S9 examples green; H4 re-run green; S6b's 006 amendments
      logged; S8 bill recorded; both ARM64 artifacts built; synth + both
      nag packs clean-or-suppressed; scan pack local runs + reconciliation
      identity recorded; S12 deploy-day evidence incl. **buckets retained,
      compute torn down**; property-auditor pass.

*Deviation declared: deploy/teardown happen in sitting U on the learner's
account (no credentials in the authoring session); the stateful stack is
intentionally NOT torn down — it is the lake.*

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-10 | Initial fast-path draft | Part-6 directive (full loop) | pending combined ack |
| 2026-07-10 | Rev 2.1 (materials reconciliation): S8 split into predict (sitting S) / measure (sitting T) — the ingest binary can't be weighed before the sink swap exists | sitting-guide authoring | this combined gate |
| 2026-07-10 | Rev 2 per audits (reqs 68%, design+tasks 58%): 1.1 gains the lake-store crate + the planned Send wall; generator domain includes traces//bad-ts; 1.4 gauge-inside-put + fail_on; 1.5 pauses at the quote-strip; 1.6 carries the S6b change-protocol amendment of 006; ops checklist updated to match | 007 audits | this combined gate |

</details>

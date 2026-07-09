# ⚠️ Reference implementation — don't peek until you've tried

This is the **course answer key** for spec 003. It exists so that every
worksheet, checkpoint, and test in the course is *validated* — if it's on the
trail, it compiles and passes here first.

**Learner contract:** attempt each sitting from its worksheet
(`../sittings/`) first. Come here only when you're stuck *after* trying and
after the worksheet's hint ladder — and even then, read only the one function
you're stuck on. Your own glake grows in `crates/glake`; this copy is never
imported by anything.

Layout mirrors the design doc: `scan.rs` (the heart), `classify.rs`,
`walk.rs`, `tally.rs`, thin `main.rs`. Tests cover R1–R10 including both
property suites (R8, R9) and the schema drift test.

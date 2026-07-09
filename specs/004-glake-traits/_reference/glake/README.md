# ⚠️ Reference implementation — don't peek until you've tried

This is the **course answer key** for spec 004 (glake v1). It exists so that
every worksheet, checkpoint, and test in the course is *validated* — if it's
on the trail, it compiles and passes here first. It started as a copy of the
003 reference (`../../003-rust-bedrock/_reference/glake`, frozen there as the
v0 answer key) and evolved exactly the way sittings G–J walk you through.

**Learner contract:** attempt each sitting from its worksheet
(`../sittings/`) first. Come here only when you're stuck *after* trying and
after the worksheet's hint ladder — and even then, read only the one function
you're stuck on. Your own glake grows in `crates/glake`; this copy is never
imported by anything.

Layout mirrors the 004 design doc — v0's modules survive, v1 adds four:

| Module | Sitting | What it teaches |
|---|---|---|
| `scan.rs`, `classify.rs` | C/D (003) | the zero-copy core (day rule tightened for F2) |
| `walk.rs`, `tally.rs` | B/E (003) → H/I | now `GlakeError`-threaded; `tally_filtered` is the F13 generic pipeline |
| `error.rs` | H | `thiserror`, source chains, C-GOOD-ERR |
| `cli.rs` | I | clap derive; filters are stats-only; `--parser` seam |
| `filter.rs` | I | closures + iterator adapters; the `Verdict` three-way |
| `parser.rs` | J | the `EventParser` trait, `HandParser` vs `SerdeParser`, the owned boundary |

Tests cover the carried-over 003 suites (R8, R9, schema drift, CLI) plus
v1's: `prop_filter.rs` ([P] F3 partition), `prop_parsers.rs` ([P] F8a exact
equivalence + F8b divergence classification). Known, documented hand/serde
divergences live in `../NOTES.md`.

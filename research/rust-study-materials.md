# Rust Study Materials — for this learner

**Researched:** 2026-07-05 · **Method:** deep-research workflow, **interrupted**
(Fable-5 rate limit hit mid-verification: 34/103 agents completed, synthesis
skipped). Per protocol v2's no-silent-drops rule, salvageable claims are kept
below with **honest verification labels** — 2 reached a 3-0 vote, the rest are
**sourced + source-quoted but verification-incomplete (rate-limited, not
refuted)**. Treat the latter as strong leads, not settled facts.

**Learner fit:** AWS-expert, Rust-beginner, build-first + hands-on preference,
10+ h/week, bilingual EN/KR. So: exercises woven into bolts as warm-ups (his
stated preference), reading kept optional, async deferred until after
ownership/traits/errors (every source agrees), and the memlens tool substitutes
for the ownership-visualization that beginners most need.

## The material set (ranked by leverage for this profile)

### Tier 1 — bind as spec warm-ups (hands-on, free)
- **Rustlings** *(sourced)* — official rust-lang exercises, designed to run
  *alongside* The Book. Perfect bolt warm-ups; map its topic sets to each spec.
  <https://github.com/rust-lang/rustlings/>
- **100 Exercises to Learn Rust** (Mainmatter) *(sourced)* — 100 progressively
  harder exercises, zero-prior-experience, **concurrency sequenced last**
  (ch07 threads → ch08 async). Its ordering validates our 003→005 arc.
  <https://rust-exercises.com/100-exercises/>

### Tier 2 — reference/optional reading
- **The Rust Book, Brown interactive edition** *(3-0 confirmed on what it is;
  sourced on the rest)* — experimental fork by Brown's Cognitive Engineering
  Lab with **retryable quizzes** and an **Aquascope-diagram ownership chapter**,
  backed by a peer-reviewed learning study. Two findings that shape *our* plan:
  drop-out concentrates at the **ownership chapters** (→ front-load support
  there — that's exactly what memlens does), and **"why" questions beat
  "does-it-compile"** questions (→ our error-driven, explain-the-borrow-checker
  approach is evidence-backed). <https://rust-book.cs.brown.edu/>
- **Comprehensive Rust** (Google) *(sourced)* — assumes programming experience,
  not Rust; async split into a separate day *after* fundamentals. Good for
  concentrated reading; **has a complete Korean translation** at `/ko/`.
  <https://google.github.io/comprehensive-rust/>
- **RustBooks list** *(sourced)* tiering, for later: **starter** = *Zero To
  Production* (directly relevant — axum/tokio/production patterns, pairs with
  spec 005), *Command-Line Rust* (pairs with spec 003's CLI). **Advanced/defer
  to Phase 2+**: *Programming Rust*, *Rust for Rustaceans*, *Effective Rust*,
  *Rust Atomics and Locks*, *Rustonomicon*. <https://github.com/sger/RustBooks>

### Async guidance (spec 005) — cloud-engineer pitfalls, from Tokio's own docs *(sourced)*
- **Don't reach for async reflexively**: Tokio docs say *not* to use it for
  CPU-bound work (use Rayon), many-file reads, or a single request. Counters the
  "everything network = async" instinct.
- **`Arc<Mutex>` everywhere is a smell**: prefer `std::sync::Mutex` for
  low-contention shared state; **never hold a lock across `.await`** (the std
  guard isn't `Send`; a blocking lock stalls the runtime thread). This becomes
  an explicit anti-pattern checklist in spec 005.
- *Async Rust* (Flitton & Morton) and the Tokio tutorial both assume
  fundamentals first — post-005 reading, not entry points.

### Korean-language resources *(sourced)*
- **doc.rust-kr.org** — de-facto Korean translation of The Book (2021 edition,
  w/ J-Pub); lags English (no Brown/2024 content) — use as a comprehension aid,
  not the primary. **rust-kr.org Discord** is a live native-language Q&A venue.
  Korean *Rust By Example* and *Tour of Rust* also exist.

## What to skip (for now)
Rustonomicon (except the one memlens `unsafe` lesson, already used), Atomics and
Locks, Rust for Rustaceans, video-only courses (weak retention for a build-first
learner) — all deferred to Phase 2+ per the RustBooks tiering.

## Material → spec pairing (Phase 1)
| Spec | Warm-up (required) | Optional reading |
|---|---|---|
| 003 rust-bedrock | Rustlings: variables→enums→error_handling→iterators; 100-Exercises ch1–4 | Book ch3–6 (Brown), Command-Line Rust |
| 004 glake-traits | Rustlings: traits, generics, modules; 100-Exercises ch5–6 | Book ch10, API Guidelines |
| 005 async-relay | 100-Exercises ch7–8; Tokio tutorial §1–6 | Zero To Production ch1–7 |
| 006 hello-lambda | (project is the exercise) | AWS Rust-on-Lambda blog |
| 007 lake-to-s3 | (project is the exercise) | aws-sdk-rust examples |
| 008 lambda-memlab | (capstone) | — |

## To finish this research (Phase 1.5, when credits allow)
Re-run `research/rust-study-materials` with resumeFromRunId to complete
verification of the ~30 rate-limited claims and add: Exercism track, Gjengset
video series depth, dtolnay Rust Quiz, spaced-repetition evidence, exemplary
crates to *read* (std/serde/tokio/axum). Backlog per protocol v2.

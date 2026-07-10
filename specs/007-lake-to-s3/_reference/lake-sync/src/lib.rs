//! lake-sync — walk the local lake, upload what's missing (spec 007
//! reference, v0).
//!
//! ```text
//!   local lake ─► glake::walk ─► plan (pure: keys + size/etag compare)
//!                                  │
//!                        --dry-run ┤ print & stop (no store, no AWS)
//!                                  ▼
//!                                run (JoinSet + Semaphore(4) over Arc<S>)
//!                                  ▼
//!                        S: ObjectStore  ── FakeStore in tests
//!                                        └─ S3Store   in main
//! ```
//!
//! Pure-core / impure-shell, the same layout every crate in this course
//! has worn: [`plan`] decides everything and touches no network;
//! [`run`] executes the decisions against any [`lake_store::ObjectStore`].
//! The properties (tests/prop_sync.rs) drive both against the fake at 512
//! generated lakes; `main.rs` is the only file that knows AWS exists.
//!
//! Idempotence works like the lake itself — by content, not memory: LIST
//! the `raw/` prefix once, compare each local file's size (cheap, first)
//! and md5-etag (second), upload only the differing. Safe-to-run-twice is
//! an architecture property here, not a flag (T5).

#![forbid(unsafe_code)]

pub mod plan;
pub mod run;

/// Everything that can go wrong in the library. Two variants of our own
/// plus transparent passthrough of the walker's — resist inventing error
/// taxonomy before there are callers who would match on it (004's lesson).
///
/// `main` maps ANY of these to one stderr line (`lake-sync: <msg>`) and
/// exit 2 — they are all "could not even try" failures (usage/io), as
/// opposed to per-object put failures, which are DATA (reported in the
/// [`run::Outcome`], partial progress and all — S5).
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    /// The walker failed (missing/unreadable path, with path context) —
    /// glake's error Display is already user-grade, so it passes through
    /// verbatim rather than being re-wrapped into noise.
    #[error(transparent)]
    Walk(#[from] glake::error::GlakeError),

    /// Reading a local file failed mid-plan or mid-run. Carries the path.
    #[error("cannot read {path}: {source}")]
    Io {
        /// The file we were reading.
        path: std::path::PathBuf,
        /// The underlying io error, chained as `source()`.
        #[source]
        source: std::io::Error,
    },

    /// The remote LIST failed (real store only; the fake never fails it).
    #[error(transparent)]
    Store(#[from] lake_store::StoreError),

    /// A lake path that isn't valid UTF-8 cannot become an S3 key
    /// honestly. The real lake's paths are ASCII (`dt=YYYY-MM-DD/…`);
    /// refusing loudly beats uploading a lossily-renamed object.
    #[error("path is not valid utf-8: {}", path.display())]
    NonUtf8Path {
        /// The offending path.
        path: std::path::PathBuf,
    },
}

//! lake-store — one trait is the whole architecture (spec 007 reference).
//!
//! ```text
//!                     ┌─────────────── lake-store ───────────────────────┐
//!                     │  ObjectStore · ObjectMeta · StoreError           │
//!                     │  FakeStore  (feature "fake": BTreeMap + ledger)  │
//!                     │  S3Store    (feature "s3":  aws-sdk-s3)          │
//!                     └───────┬──────────────────────────┬───────────────┘
//!               lake-sync ────┘                          └──── hello-lambda
//!               (CLI: plan + run)                              (ingest: door → put)
//! ```
//!
//! This is 004's trait lesson graduating into a **testing strategy**: put
//! the seam where the un-testable thing starts. Everything interesting —
//! planning uploads, idempotence, bounded concurrency, the conservation
//! law — happens in code that has never heard of AWS, property-tested
//! against [`fake::FakeStore`]; the real [`s3::S3Store`] is a translation
//! layer thin enough to read in one sitting.
//!
//! Contrast with 004's seam: glake's `--parser` flag needed a *runtime*
//! choice, so it took `Box<dyn EventParser>`. Here the implementation is
//! chosen at **compile time** (prod binaries name `S3Store`, tests name
//! `FakeStore`), so consumers stay generic over `S: ObjectStore` and
//! monomorphize — the F13 pattern's second appearance, no `dyn` anywhere.
//! (RPITIT methods also aren't dyn-friendly without boxing helpers, so the
//! generic shape is the honest one twice over.)

#![forbid(unsafe_code)]

use std::future::Future;

#[cfg(feature = "fake")]
pub mod fake;
#[cfg(feature = "s3")]
pub mod s3;

/// What a store knows about one object it holds — the compare side of the
/// idempotence check (size first because it's free, etag second).
///
/// Derives per the C-COMMON-TRAITS rubric pass (S7): `Debug` (test
/// output), `Clone` (metas outlive the listing they came from),
/// `PartialEq`/`Eq` + `Hash` (tests compare and set-ify them). Not `Copy`
/// (owns `String`s), not `Ord` (no natural total order worth promising).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectMeta {
    /// Full key, e.g. `raw/dt=2026-07-09/events.jsonl`.
    pub key: String,
    /// Object size in bytes — the cheap first line of the compare.
    pub size: u64,
    /// Entity tag as **bare lowercase hex, no quotes** — for a single-part
    /// PUT under SSE-S3 this is the md5 of the body, which is what makes
    /// `uploaded 0` provable. Both impls promise this exact spelling:
    /// `S3Store::list` strips the SDK's wrapping quotes, `FakeStore::list`
    /// computes `format!("{:x}", md5)`. See s3.rs for where the
    /// etag-is-md5 premise breaks (SSE-KMS, multipart).
    pub etag: String,
}

/// The seam. Implemented twice: [`fake::FakeStore`] for the laws,
/// [`s3::S3Store`] for reality.
///
/// ## Why this is NOT written `async fn` (the lesson, taken on purpose)
///
/// The sugar `async fn list(&self, ...)` desugars to exactly the RPITIT
/// form below **minus the `+ Send`** — and on stable Rust there is no way
/// to bolt a Send bound onto the sugar. Consumers spawn these futures
/// (`tokio::task::JoinSet::spawn` in lake-sync; lambda_http's service
/// bounds in hello-lambda), and `spawn` demands `Send` because the runtime
/// may migrate a task between worker threads. With the sugar, the trait
/// compiles and every *use site* fails. Writing the desugared
/// `impl Future<Output = …> + Send` by hand is the async-fn-in-trait
/// lesson itself; a proc-macro (`trait_variant`) would do the same thing
/// while hiding it.
///
/// The supertraits carry the other half of the spawn contract: the store
/// itself crosses threads inside an `Arc` (`Send + Sync`) and outlives
/// every spawned task (`'static`).
pub trait ObjectStore: Send + Sync + 'static {
    /// Every object whose key starts with `prefix`, in one shot (the real
    /// impl drains the pagination for you — callers never see pages).
    fn list(
        &self,
        prefix: &str,
    ) -> impl Future<Output = Result<Vec<ObjectMeta>, StoreError>> + Send;

    /// Store `body` at `key`, **replacing** whatever was there — S3 has
    /// one object per key and no append; a re-put overwrites. Owned
    /// parameters on purpose: the caller moves the key and body into a
    /// spawned task, so borrowing here would demand lifetimes no spawn
    /// can honor.
    fn put(
        &self,
        key: String,
        body: Vec<u8>,
    ) -> impl Future<Output = Result<(), StoreError>> + Send;
}

/// What can go wrong at the seam — one variant per operation, each
/// carrying the *name* the caller needs (C-GOOD-ERR: lowercase message,
/// no trailing period, cause chained through `source()`, and the type is
/// `Send + Sync + 'static` so it crosses task boundaries).
///
/// The underlying causes are boxed: the fake fails with a plain string,
/// the real store with a many-generic SDK error — a `Box<dyn Error>`
/// source is the honest common shape, and nobody matches on an S3 error's
/// internals at this seam anyway.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// A LIST failed. Carries the prefix we asked about.
    #[error("cannot list objects under {prefix:?}: {source}")]
    List {
        /// The prefix the listing was scoped to.
        prefix: String,
        /// The underlying cause, reachable via `source()`.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// A PUT failed. Carries the key — S5 needs the failed object *named*.
    #[error("cannot put object {key:?}: {source}")]
    Put {
        /// The key we were writing.
        key: String,
        /// The underlying cause, reachable via `source()`.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl StoreError {
    /// Attach prefix context to a list failure (the `map_err` companion).
    pub fn list(prefix: &str, source: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        StoreError::List {
            prefix: prefix.to_owned(),
            source: source.into(),
        }
    }

    /// Attach key context to a put failure.
    pub fn put(key: &str, source: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        StoreError::Put {
            key: key.to_owned(),
            source: source.into(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::error::Error as _;

    /// C-GOOD-ERR: usable across threads, boxable, 'static — compile-time.
    #[test]
    fn s7_error_is_send_sync_static() {
        fn assert_good_err<T: std::error::Error + Send + Sync + 'static>() {}
        assert_good_err::<StoreError>();
    }

    /// C-GOOD-ERR display style: lowercase start, no trailing period, and
    /// each variant leads with the name the caller needs.
    #[test]
    fn s7_display_is_lowercase_no_period_with_context() {
        let list = StoreError::list("raw/", "connection refused");
        let put = StoreError::put("raw/dt=2026-07-09/events.jsonl", "access denied");
        for e in [&list, &put] {
            let msg = e.to_string();
            assert!(!msg.starts_with(char::is_uppercase), "lowercase: {msg}");
            assert!(!msg.ends_with('.'), "no trailing period: {msg}");
        }
        assert!(
            list.to_string()
                .starts_with("cannot list objects under \"raw/\":")
        );
        assert!(put.to_string().contains("raw/dt=2026-07-09/events.jsonl"));
    }

    /// C-GOOD-ERR: the cause is chained through source(), not smashed
    /// into the message and lost.
    #[test]
    fn s7_source_chain_reaches_the_cause() {
        let e = StoreError::put("k", "the disk is on fire");
        assert!(e.source().unwrap().to_string().contains("on fire"));
    }
}

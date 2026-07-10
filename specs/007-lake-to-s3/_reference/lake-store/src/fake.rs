//! The in-memory store the laws run against (feature `fake` — consumers
//! enable it in `[dev-dependencies]` ONLY, so nothing test-shaped can leak
//! into a deployable artifact).
//!
//! Three jobs, all in service of making side effects property-testable
//! (T2):
//!
//! 1. **Model S3's semantics where the laws lean on them.** One object per
//!    key, and a re-put **replaces** — `BTreeMap::insert`, which returns
//!    the old value and stores the new, is exactly that semantic. This is
//!    what keeps the S3 conservation law true across re-syncs: a changed
//!    file re-uploads and the store holds one current copy, not two.
//! 2. **Keep a ledger.** Counters a test can read: how many puts happened
//!    (S2's "dry-run performs zero", S4's "second sync performs zero"),
//!    *which* keys they were (S4's "exactly the changed set"), and an
//!    in-flight gauge with a high-water mark whose increment/decrement
//!    live INSIDE `put` itself — so S9's concurrency bound is measured
//!    where the work happens, not where it was scheduled.
//! 3. **Inject failure.** [`FakeStore::fail_on`] makes `put` fail for one
//!    chosen key — S5's mid-run-failure behavior (name the object, report
//!    partial progress, exit non-zero) needs a store that CAN fail on cue.
//!
//! Interior mutability, the honest way: the trait's methods take `&self`
//! (many tasks share one store through an `Arc`), so every mutable thing
//! here lives in a `Mutex` or an atomic — the same shape relay's counters
//! took in 005, now as a *test instrument*.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::{ObjectMeta, ObjectStore, StoreError};

/// An in-memory [`ObjectStore`] with a ledger. Build one per test; read
/// the ledger after driving the code under test.
#[derive(Debug, Default)]
pub struct FakeStore {
    /// The "bucket": key → body. `BTreeMap` (not `HashMap`) so `list`
    /// comes back sorted like S3's lexicographic listings — determinism
    /// for free.
    objects: Mutex<BTreeMap<String, Vec<u8>>>,
    /// If set, `put` fails for exactly this key (S5 injection point).
    fail_on: Mutex<Option<String>>,
    /// Successful puts, in order — the ledger's "what changed" answer.
    put_log: Mutex<Vec<String>>,
    /// Count of successful puts (== put_log.len(), kept as an atomic so
    /// tests can read it without locking).
    puts: AtomicU64,
    /// How many `put` calls are between their entry and exit right now.
    in_flight: AtomicU64,
    /// The highest value `in_flight` ever reached — S9's witness.
    high_water: AtomicU64,
}

/// Lock helper: a poisoned mutex means another test thread panicked while
/// holding it; the data (test bookkeeping) is still fine to read, and
/// propagating the poison would only turn one failure into two. This is
/// the standard `unwrap_or_else(PoisonError::into_inner)` recovery — and
/// it keeps the crate's unwrap ban intact.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl FakeStore {
    /// A fresh, empty store with a zeroed ledger.
    pub fn new() -> Self {
        FakeStore::default()
    }

    /// Make the NEXT and every later `put` of `key` fail (until cleared).
    /// The failed put stores nothing and is not counted in the ledger —
    /// exactly like a real PUT that dies on the wire.
    pub fn fail_on(&self, key: impl Into<String>) {
        *lock(&self.fail_on) = Some(key.into());
    }

    /// Number of puts that *succeeded* since construction.
    pub fn puts(&self) -> u64 {
        self.puts.load(Ordering::SeqCst)
    }

    /// Every successfully put key, in completion order. Tests diff this
    /// across passes to prove "exactly the changed set uploaded" (S4).
    pub fn put_log(&self) -> Vec<String> {
        lock(&self.put_log).clone()
    }

    /// The most `put`s that were ever simultaneously in flight (S9).
    pub fn high_water(&self) -> u64 {
        self.high_water.load(Ordering::SeqCst)
    }

    /// One object's current body, if present.
    pub fn object(&self, key: &str) -> Option<Vec<u8>> {
        lock(&self.objects).get(key).cloned()
    }

    /// Snapshot of the whole bucket (key → body) for whole-lake
    /// assertions like the conservation multiset.
    pub fn objects(&self) -> BTreeMap<String, Vec<u8>> {
        lock(&self.objects).clone()
    }
}

// teach: the TRAIT spells `impl Future + Send` by hand because the sugar
// can't promise Send — but an IMPL may use the sugar. The compiler proves
// each concrete future here IS Send (auto-traits leak through `async fn`)
// and checks that against the trait's written promise; if `put` ever held
// a `MutexGuard` across one of its `.await`s, this impl would stop
// compiling. (clippy::manual_async_fn points the same direction: desugar
// only where the desugaring SAYS something — the trait — not everywhere.)
impl ObjectStore for FakeStore {
    async fn list(&self, prefix: &str) -> Result<Vec<ObjectMeta>, StoreError> {
        // Etags are computed AT LIST TIME from the stored bytes, in
        // the same bare-hex md5 spelling S3Store::list yields after
        // quote-stripping — the fake and reality cannot drift on
        // format, because both sides of every compare go through
        // `format!("{:x}", …)` on 16 raw digest bytes.
        Ok(lock(&self.objects)
            .iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .map(|(key, body)| ObjectMeta {
                key: key.clone(),
                size: body.len() as u64,
                etag: format!("{:x}", md5::compute(body)),
            })
            .collect())
    }

    async fn put(&self, key: String, body: Vec<u8>) -> Result<(), StoreError> {
        // The gauge brackets the WHOLE body of the call — this is the
        // "measured inside put, not where it was scheduled" rule: a
        // scheduler-side gauge would count queued tasks that haven't
        // started, and S9's bound is about work in flight.
        let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.high_water.fetch_max(now, Ordering::SeqCst);

        // Dwell across two yield points so "in flight" is OBSERVABLE:
        // a put that never awaits would start and finish within one
        // poll, other tasks could never overlap it, and the S9 gauge
        // test would pass vacuously with a high-water mark of 1. A
        // real network PUT parks the task at .await exactly like this;
        // yield_now is the deterministic lab version of that parking.
        tokio::task::yield_now().await;

        let result = if lock(&self.fail_on).as_deref() == Some(key.as_str()) {
            Err(StoreError::put(
                &key,
                "injected failure (FakeStore::fail_on)",
            ))
        } else {
            // `insert` REPLACES any previous body at this key — S3's
            // one-object-per-key semantic, which the S3/S4 laws lean
            // on (a re-synced changed file must overwrite, not pile up).
            lock(&self.objects).insert(key.clone(), body);
            lock(&self.put_log).push(key);
            self.puts.fetch_add(1, Ordering::SeqCst);
            Ok(())
        };

        tokio::task::yield_now().await;
        // Decrement on BOTH paths — a failed put was still in flight.
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
        result
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// The semantic the laws lean on: a re-put REPLACES; one object per key.
    #[tokio::test]
    async fn put_replaces_the_object_at_a_key() {
        let store = FakeStore::new();
        store.put("raw/a".into(), b"old".to_vec()).await.unwrap();
        store.put("raw/a".into(), b"new".to_vec()).await.unwrap();
        assert_eq!(store.object("raw/a"), Some(b"new".to_vec()));
        assert_eq!(store.objects().len(), 1, "one object per key");
        assert_eq!(store.puts(), 2, "both puts counted");
        assert_eq!(
            store.put_log(),
            vec!["raw/a".to_owned(), "raw/a".to_owned()]
        );
    }

    /// The etag spelling contract: bare-hex md5 of the STORED bytes.
    /// (d41d… is the famous md5 of the empty input — pinned so a format
    /// change can't hide.)
    #[tokio::test]
    async fn list_computes_bare_hex_md5_etags() {
        let store = FakeStore::new();
        store.put("raw/empty".into(), Vec::new()).await.unwrap();
        store.put("raw/hi".into(), b"hi\n".to_vec()).await.unwrap();
        store.put("other/x".into(), b"x".to_vec()).await.unwrap();

        let listed = store.list("raw/").await.unwrap();
        assert_eq!(listed.len(), 2, "prefix-scoped: other/x filtered out");
        assert_eq!(listed[0].key, "raw/empty");
        assert_eq!(listed[0].size, 0);
        assert_eq!(listed[0].etag, "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(listed[1].etag, format!("{:x}", md5::compute(b"hi\n")));
        assert!(
            listed.iter().all(|m| !m.etag.contains('"')),
            "bare hex, never quoted"
        );
    }

    /// The S5 injection point: the chosen key fails (named in the error),
    /// stores nothing, counts nothing; other keys are untouched.
    #[tokio::test]
    async fn fail_on_fails_exactly_the_chosen_key() {
        let store = FakeStore::new();
        store.fail_on("raw/doomed");
        let err = store
            .put("raw/doomed".into(), b"x".to_vec())
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("raw/doomed"),
            "object named: {err}"
        );
        assert!(
            store.object("raw/doomed").is_none(),
            "failed put stores nothing"
        );
        assert_eq!(store.puts(), 0, "failed put not counted");

        store.put("raw/fine".into(), b"y".to_vec()).await.unwrap();
        assert_eq!(store.puts(), 1);
        assert_eq!(
            store.in_flight.load(Ordering::SeqCst),
            0,
            "gauge balanced on both paths"
        );
    }
}

//! [P] A4/A5 — conservation and partition correctness under REAL
//! concurrency (the co-written hard bit: async × proptest).
//!
//! The pattern, in one breath: proptest drives a **sync** test body, the
//! body builds its own **multi-thread** tokio `Runtime` and `block_on`s the
//! scenario — generation stays deterministic (proptest's RNG never crosses
//! an `.await`), only the scenario is async. Inside, every request gets its
//! OWN `tokio::spawn` against a cloned router: that's what makes handlers
//! actually race on worker threads. The tempting shortcut —
//! `join_all(bodies.map(|b| app.oneshot(b)))` — polls every future on ONE
//! task: interleaved, never parallel, and A4 would be testing nothing.
//!
//! The strawman note (design, "honesty about red"): against the sitting-L
//! direct-append handler, the tearing half of A4 may well SURVIVE on Linux
//! (O_APPEND small writes are one atomic syscall — the OS quietly saves
//! it); the conservation half (shutdown-mid-queue loses nothing) is where
//! the strawman goes genuinely red. This channel version passes both **by
//! construction**: one owner, drain-then-flush.

#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashMap;

use common::{TempLake, body_json, canon, disk_lines, get_healthz, post_event, spawn_relay};
use glake::classify::{REQUIRED_KEYS, day_of_ts};
use glake::filter::Filter;
use glake::parser::SerdeParser;
use glake::tally::tally_filtered;
use glake::walk::{jsonl_files, read_file};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use tower::ServiceExt;

// ---------------------------------------------------------------- generators

/// A MULTI-DAY ts domain (the design insists): a single-day domain would
/// let a writer that dumps everything into one folder pass A5 by luck.
/// Weird-but-comparable days are in the domain on purpose — the door
/// accepts what it can partition, not what a calendar blesses.
const DAYS: [&str; 5] = [
    "2026-07-05",
    "2026-07-08",
    "2026-07-09",
    "1999-01-01",
    "2027-12-31",
];
const KINDS: [&str; 4] = [
    "session.start",
    "gate.approved",
    "bolt.done",
    "learning.fight",
];
/// ts values with no comparable day (glake's F2 rule says bad-ts).
const BAD_TS: [&str; 4] = ["nope", "2026/07/09T01:02:03Z", "today", "20260709"];
/// Bodies that aren't JSON objects at all.
const NON_OBJECTS: [&str; 6] = [
    "not json at all",
    "[1,2,3]",
    "123",
    "\"quoted\"",
    "null",
    "",
];

/// A well-formed envelope as a `Value` (all REQUIRED_KEYS present).
fn envelope(id: &str, ts: &str, kind: &str) -> serde_json::Value {
    serde_json::json!({
        "event_id": id,
        "ts": ts,
        "session_id": "prop-relay",
        "actor": "proptest",
        "event_type": kind,
        "schema_version": 1,
        "payload": { "case": id }
    })
}

fn ts() -> impl Strategy<Value = String> {
    (0..DAYS.len(), 0u32..24, 0u32..60, 0u32..60)
        .prop_map(|(d, h, m, s)| format!("{}T{h:02}:{m:02}:{s:02}Z", DAYS[d]))
}

/// A valid body — sometimes pretty-printed, so it SPANS LINES: the raw
/// body is legal JSON either way, and A4's re-serialization rule is what
/// keeps the lake one-line-per-event.
fn valid_body() -> impl Strategy<Value = String> {
    ("[a-z0-9]{6}", ts(), 0..KINDS.len(), any::<bool>()).prop_map(|(id, ts, k, pretty)| {
        let value = envelope(&id, &ts, KINDS[k]);
        if pretty {
            serde_json::to_string_pretty(&value).unwrap()
        } else {
            value.to_string()
        }
    })
}

/// The malformed arms, one per door clause (mirrors 004's malformed
/// strategies): non-objects, an envelope with one required key removed,
/// and an envelope whose ts has no comparable day.
fn invalid_body() -> impl Strategy<Value = String> {
    prop_oneof![
        (0..NON_OBJECTS.len()).prop_map(|i| NON_OBJECTS[i].to_owned()),
        ("[a-z0-9]{6}", ts(), 0..REQUIRED_KEYS.len()).prop_map(|(id, ts, k)| {
            let mut value = envelope(&id, &ts, "prop.malformed");
            value.as_object_mut().unwrap().remove(REQUIRED_KEYS[k]);
            value.to_string()
        }),
        ("[a-z0-9]{6}", 0..BAD_TS.len())
            .prop_map(|(id, t)| envelope(&id, BAD_TS[t], "prop.badts").to_string()),
    ]
}

/// One message, tagged with what the door MUST say about it — the oracle
/// travels with the input.
#[derive(Debug, Clone)]
enum Msg {
    Valid(String),
    Invalid(String),
}

impl Msg {
    fn body(&self) -> &str {
        match self {
            Msg::Valid(body) | Msg::Invalid(body) => body,
        }
    }
}

/// A mixed batch: ~3:1 valid:invalid, and occasional EXACT duplicates
/// (repeat count 2) — the no-dedup policy says duplicate submissions yield
/// duplicate lines, and multiset equality is what catches a writer that
/// "helpfully" dedups (or a HashMap-of-lines test that can't see doubles).
fn batch() -> impl Strategy<Value = Vec<Msg>> {
    let msg = prop_oneof![
        3 => valid_body().prop_map(Msg::Valid),
        1 => invalid_body().prop_map(Msg::Invalid),
    ];
    let repeat = prop_oneof![5 => Just(1usize), 1 => Just(2usize)];
    prop::collection::vec((msg, repeat), 1..16).prop_map(|pairs| {
        pairs
            .into_iter()
            .flat_map(|(msg, n)| std::iter::repeat_n(msg, n))
            .collect()
    })
}

// ------------------------------------------------------------- the scenario

/// Everything one concurrent relay session left behind. Holds the
/// `TempLake` so the on-disk evidence outlives the runtime that made it
/// (the dir is cleaned when the `Outcome` drops).
struct Outcome {
    lake: TempLake,
    /// Canonical forms of the bodies that got 202, in join order.
    accepted: Vec<String>,
    /// How many bodies got 400.
    rejected: u64,
    /// The writer's own count of lines flushed to disk.
    written: u64,
    /// The /healthz JSON, snapshotted AFTER every request joined (quiescent).
    health: serde_json::Value,
}

/// Fire `msgs` concurrently at a fresh relay over a temp lake, drain
/// gracefully, and collect the evidence. Shared verbatim by A4 and A5 —
/// same machinery, different assertions.
fn run_scenario(msgs: Vec<Msg>) -> Result<Outcome, TestCaseError> {
    // A fresh multi-thread runtime per case: worker threads are what turn
    // "spawned tasks" into "requests racing in parallel".
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .map_err(|e| TestCaseError::fail(format!("runtime build: {e}")))?;

    rt.block_on(async move {
        let lake = TempLake::new();
        let (app, writer) = spawn_relay(lake.path());

        // One tokio::spawn per request (see module docs for why join_all
        // over bare oneshot futures would be a fake). Each task owns a
        // Router clone — cheap (an Arc'd tree) and the whole point of
        // AppState: Clone.
        let mut requests = Vec::with_capacity(msgs.len());
        for msg in msgs {
            let app = app.clone();
            requests.push(tokio::spawn(async move {
                // Router's service error is Infallible, so unwrap here is
                // total, not hopeful.
                let response = app.oneshot(post_event(msg.body())).await.unwrap();
                (msg, response.status().as_u16())
            }));
        }

        // Join every request and check each against its oracle.
        let mut accepted = Vec::new();
        let mut rejected = 0u64;
        for handle in requests {
            let (msg, status) = handle
                .await
                .map_err(|e| TestCaseError::fail(format!("request task: {e}")))?;
            match msg {
                Msg::Valid(body) => {
                    prop_assert_eq!(status, 202, "valid body must 202: {}", body);
                    accepted.push(canon(&body));
                }
                Msg::Invalid(body) => {
                    prop_assert_eq!(status, 400, "invalid body must 400: {}", body);
                    rejected += 1;
                }
            }
        }

        // Quiescent by construction (every request joined) — so this
        // snapshot is exactly the world A3's invariant speaks about.
        let response = app.clone().oneshot(get_healthz()).await.unwrap();
        let health = body_json(response.into_body()).await;

        // Graceful drain: drop the LAST router handle → the last sender
        // drops → the writer's recv() yields None → drain + flush → the
        // JoinHandle yields the written count. Same choreography as
        // main.rs after ctrl-c.
        drop(app);
        let written = writer
            .await
            .map_err(|e| TestCaseError::fail(format!("writer task: {e}")))?;

        Ok(Outcome {
            lake,
            accepted,
            rejected,
            written,
            health,
        })
    })
}

// ------------------------------------------------------------ the properties

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// [P] A4 — acceptance conservation: multiset(parsed lines on disk) ==
    /// multiset(parsed bodies that got 202). Nothing torn, merged, lost, or
    /// invented; duplicates preserved; rejected bodies appear nowhere (any
    /// 400'd body on disk would break the equality as a surplus line).
    #[test]
    fn prop_a4_conservation(msgs in batch()) {
        let total = msgs.len() as u64;
        let out = run_scenario(msgs)?;

        // Multiset equality, executable form: canonicalize (parse +
        // compact re-serialize with sorted keys), then sort both sides.
        let mut on_disk: Vec<String> = disk_lines(out.lake.path())
            .into_iter()
            .map(|(_, line)| canon(&line))
            .collect();
        let mut accepted = out.accepted.clone();
        on_disk.sort();
        accepted.sort();
        prop_assert_eq!(on_disk, accepted, "disk multiset != 202'd multiset");

        // The writer's own count agrees (no double-append, no drop).
        prop_assert_eq!(out.written, out.accepted.len() as u64);

        // And the A3 invariant, observed after real concurrency:
        prop_assert_eq!(out.health["received"].as_u64(), Some(total));
        prop_assert_eq!(out.health["accepted"].as_u64(), Some(out.accepted.len() as u64));
        prop_assert_eq!(out.health["rejected"].as_u64(), Some(out.rejected));
    }

    /// [P] A5 — partition correctness: every line lives in the dt= folder
    /// its OWN ts names, and glake — the independent reader built in 004 —
    /// agrees with the relay about what landed, in total and day by day.
    #[test]
    fn prop_a5_partitions(msgs in batch()) {
        let out = run_scenario(msgs)?;

        // 1) folder == the line's own comparable day, line by line.
        for (folder_day, line) in disk_lines(out.lake.path()) {
            let value: serde_json::Value = serde_json::from_str(&line)
                .map_err(|e| TestCaseError::fail(format!("unparseable lake line {line:?}: {e}")))?;
            let ts = value["ts"].as_str().unwrap_or("<non-string>");
            prop_assert_eq!(
                day_of_ts(ts), Some(folder_day.as_str()),
                "line in dt={} has ts {}", folder_day, ts
            );
        }

        // 2) the glake cross-check: walk the temp lake with the 004 lib's
        // own walker and tally pipeline. Its event total must equal the
        // relay's accepted count, it must see zero malformed lines (the
        // strict door kept them out), and its by-day map must match the
        // days the 202'd bodies claimed.
        let files = jsonl_files(out.lake.path())
            .map_err(|e| TestCaseError::fail(format!("glake walk: {e}")))?;
        let contents: Vec<String> = files
            .iter()
            .map(|f| read_file(f).map_err(|e| TestCaseError::fail(format!("glake read: {e}"))))
            .collect::<Result<_, _>>()?;
        let stats = tally_filtered(
            &SerdeParser,
            &REQUIRED_KEYS,
            contents.iter().flat_map(|c| c.lines()),
            &Filter::default(),
        );
        prop_assert_eq!(stats.kept.events, out.accepted.len() as u64, "glake total != accepted");
        prop_assert_eq!(stats.kept.malformed, 0, "strict door let a malformed line through");

        let mut expected_by_day: HashMap<String, u64> = HashMap::new();
        for body in &out.accepted {
            let value: serde_json::Value = serde_json::from_str(body).unwrap();
            let day = day_of_ts(value["ts"].as_str().unwrap()).unwrap();
            *expected_by_day.entry(day.to_owned()).or_insert(0) += 1;
        }
        prop_assert_eq!(stats.kept.by_day, expected_by_day, "glake by-day map disagrees");
    }
}

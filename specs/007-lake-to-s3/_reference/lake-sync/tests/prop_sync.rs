//! [P] S3 + [P] S4 — the laws, run against the FAKE (that's the point,
//! T2): 512 generated lakes per property (design floor 256), each a REAL
//! tempdir of files the sync walks with the real walker — the properties
//! exercise every line of plan + run except the aws-sdk translation
//! layer, which is thin enough to be covered by compile + review + deploy
//! day (design.md, "Properties").
//!
//! The generator's domain matches S1's REAL domain, not a sanitized one:
//! `dt=` days INCLUDING a `dt=bad-ts` arm (relay's quarantine partition
//! is legal lake content) and a `traces/dt=…` arm (the volume workload's
//! real shape); file contents include blanks and garbage because sync
//! moves *bytes*, not judgments — a malformed line is exactly as worth
//! conserving as a valid one.
//!
//! Runtime pattern (005's): proptest drives a sync body; the body builds
//! its own current-thread runtime and `block_on`s, so generation stays
//! deterministic and nothing here needs to race.

#![allow(clippy::unwrap_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

use lake_store::fake::FakeStore;
use lake_sync::plan::{Plan, plan};
use lake_sync::run::{Outcome, run};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

// ------------------------------------------------------------- generators

/// Partition dirs, matching S1's real domain: plain days, the bad-ts
/// quarantine, and the traces arm. (dt=2026-07-07 is deliberately absent —
/// the S4 new-file mutation claims it, collision-free.)
const DIRS: [&str; 6] = [
    "dt=2026-07-05",
    "dt=2026-07-06",
    "dt=2026-07-09",
    "dt=bad-ts",
    "traces/dt=2026-07-05",
    "traces/dt=2026-07-08",
];

/// File names the walker accepts (it only picks up `.jsonl`).
const NAMES: [&str; 4] = [
    "events.jsonl",
    "spill-01.jsonl",
    "run-a.jsonl",
    "extra.jsonl",
];

/// One lake line: mostly valid envelopes, plus blanks and garbage —
/// sync must conserve ALL of them, verbatim. (No `\n` in any arm: lines
/// are joined by the file builder.)
fn line() -> impl Strategy<Value = String> {
    prop_oneof![
        4 => ("[a-z0-9]{1,8}", "[a-z]{3,8}").prop_map(|(id, kind)| format!(
            r#"{{"event_id":"{id}","ts":"2026-07-09T01:02:03Z","session_id":"prop","actor":"prop","event_type":"prop.{kind}","schema_version":1,"payload":{{}}}}"#
        )),
        1 => Just(String::new()),                 // blank line
        1 => Just("   \t ".to_owned()),           // whitespace line
        1 => Just("not json at all".to_owned()),  // junk
        1 => Just(r#"{"half": tru"#.to_owned()),  // truncated json
        1 => "[a-z0-9 .:{}-]{0,24}",              // arbitrary junk
    ]
}

/// A generated lake: relative path → exact file bytes. 1–8 files across
/// 1–4-ish dirs (collisions on (dir, name) fold into the map, mirroring a
/// filesystem), 0–40 lines each, trailing newline sometimes absent.
#[derive(Debug, Clone)]
struct GenLake {
    files: BTreeMap<String, Vec<u8>>,
}

fn gen_lake() -> impl Strategy<Value = GenLake> {
    prop::collection::vec(
        (
            0..DIRS.len(),
            0..NAMES.len(),
            prop::collection::vec(line(), 0..=40),
            any::<bool>(), // trailing newline?
        ),
        1..=8,
    )
    .prop_map(|files| {
        let mut map = BTreeMap::new();
        for (dir, name, lines, trailing_newline) in files {
            let mut content = lines.join("\n");
            if trailing_newline && !content.is_empty() {
                content.push('\n');
            }
            map.insert(
                format!("{}/{}", DIRS[dir], NAMES[name]),
                content.into_bytes(),
            );
        }
        GenLake { files: map }
    })
}

/// The S4 mutation arms. `SameSize` is the one that keeps the etag branch
/// of the compare load-bearing: it changes bytes WITHOUT changing size,
/// so only the checksum can notice.
#[derive(Debug, Clone)]
enum Mutation {
    Append(String),
    SameSize,
    NewFile(Vec<String>),
}

fn mutation() -> impl Strategy<Value = Mutation> {
    prop_oneof![
        line().prop_map(Mutation::Append),
        Just(Mutation::SameSize),
        prop::collection::vec(line(), 0..=10).prop_map(Mutation::NewFile),
    ]
}

// -------------------------------------------------------------- plumbing

fn write_lake(lake: &GenLake) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (rel, bytes) in &lake.files {
        let path = dir.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    dir
}

/// One full sync pass: LIST → plan → run, exactly main.rs's real path
/// with the fake in the store seat.
async fn sync(root: &Path, store: &Arc<FakeStore>) -> Result<(Plan, Outcome), TestCaseError> {
    let remote = lake_store::ObjectStore::list(store.as_ref(), "raw/")
        .await
        .map_err(|e| TestCaseError::fail(format!("fake list cannot fail: {e}")))?;
    let the_plan = plan(root, "raw/", &remote)
        .map_err(|e| TestCaseError::fail(format!("plan failed: {e}")))?;
    let outcome = run(Arc::clone(store), the_plan.clone())
        .await
        .map_err(|e| TestCaseError::fail(format!("run failed: {e}")))?;
    Ok((the_plan, outcome))
}

/// The conservation law's left/right sides: line multiset over a set of
/// bodies (order within and across files deliberately ignored — the law
/// is about *content survival*, not layout).
fn line_multiset<'a>(bodies: impl IntoIterator<Item = &'a [u8]>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for body in bodies {
        for l in std::str::from_utf8(body).unwrap().lines() {
            *counts.entry(l.to_owned()).or_insert(0) += 1;
        }
    }
    counts
}

/// Read the lake back OFF DISK (not from the generator's map) — after a
/// mutation, disk is the truth the law must hold against.
fn disk_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    glake::walk::jsonl_files(root)
        .unwrap()
        .into_iter()
        .map(|path| {
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .components()
                .map(|c| c.as_os_str().to_str().unwrap())
                .collect::<Vec<_>>()
                .join("/");
            (rel, std::fs::read(&path).unwrap())
        })
        .collect()
}

/// Assert S3's equality: store contents ≡ local lake (keys AND the line
/// multiset — nothing lost, duplicated, or invented).
fn assert_conservation(
    local: &BTreeMap<String, Vec<u8>>,
    store: &Arc<FakeStore>,
) -> Result<(), TestCaseError> {
    let stored = store.objects();
    let expected_keys: BTreeSet<String> = local.keys().map(|rel| format!("raw/{rel}")).collect();
    let stored_keys: BTreeSet<String> = stored.keys().cloned().collect();
    prop_assert_eq!(stored_keys, expected_keys, "key set drifted");
    prop_assert_eq!(
        line_multiset(stored.values().map(Vec::as_slice)),
        line_multiset(local.values().map(Vec::as_slice)),
        "line multiset drifted"
    );
    Ok(())
}

/// Apply one mutation to the on-disk lake; returns the relative path of
/// the (one) changed or created file.
fn apply_mutation(
    root: &Path,
    lake: &GenLake,
    target: &prop::sample::Index,
    mutation: &Mutation,
) -> String {
    let rels: Vec<&String> = lake.files.keys().collect();
    let rel = rels[target.index(rels.len())].clone();
    let path = root.join(&rel);
    match mutation {
        Mutation::Append(l) => {
            let mut bytes = std::fs::read(&path).unwrap();
            append_line(&mut bytes, l);
            std::fs::write(&path, bytes).unwrap();
            rel
        }
        Mutation::SameSize => {
            let mut bytes = std::fs::read(&path).unwrap();
            // Swap one alphanumeric ASCII byte for a different one: same
            // length, same UTF-8 validity, no newline structure change —
            // only the CHECKSUM can see this edit (the point of the arm).
            match bytes.iter().position(u8::is_ascii_alphanumeric) {
                Some(i) => {
                    bytes[i] = if bytes[i] == b'z' { b'y' } else { b'z' };
                    std::fs::write(&path, bytes).unwrap();
                    rel
                }
                None => {
                    // A file with no alphanumeric byte (all blanks) has no
                    // safe equal-length swap; degrade to the append arm.
                    append_line(&mut bytes, "z");
                    std::fs::write(&path, bytes).unwrap();
                    rel
                }
            }
        }
        Mutation::NewFile(lines) => {
            // dt=2026-07-07 is reserved for this arm (absent from DIRS),
            // so the new file NEVER collides with a generated one.
            let rel = "dt=2026-07-07/added.jsonl".to_owned();
            let path = root.join(&rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let mut bytes = lines.join("\n").into_bytes();
            if !bytes.is_empty() {
                bytes.push(b'\n');
            }
            std::fs::write(&path, bytes).unwrap();
            rel
        }
    }
}

/// Append `line` as its own line, whatever the file's current newline
/// posture — always changes the file by at least one byte.
fn append_line(bytes: &mut Vec<u8>, line: &str) {
    if !bytes.is_empty() && bytes.last() != Some(&b'\n') {
        bytes.push(b'\n');
    }
    bytes.extend_from_slice(line.as_bytes());
    bytes.push(b'\n');
}

// ------------------------------------------------------------ properties

proptest! {
    // 512 cases per property (design floor 256).
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// [P] S3 — sync conservation: ∀ generated lakes, after sync the
    /// multiset of lines across the fake store's objects equals the
    /// multiset across the local files — nothing lost, duplicated, or
    /// invented. (Key-set equality asserted alongside: right content at
    /// wrong keys is also a failure.)
    #[test]
    fn prop_s3_sync_conservation(lake in gen_lake()) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| TestCaseError::fail(format!("runtime build: {e}")))?;
        rt.block_on(async {
            let dir = write_lake(&lake);
            let store = Arc::new(FakeStore::new());
            let (_, outcome) = sync(dir.path(), &store).await?;
            prop_assert!(outcome.failed.is_empty(), "no injected failures here");
            assert_conservation(&lake.files, &store)?;
            Ok(())
        })?;
    }

    /// [P] S4 — idempotence: an immediate second sync performs ZERO puts;
    /// after one mutation, the third sync uploads EXACTLY the changed key
    /// and no others (the fake's put-log is the witness); and S3's
    /// equality holds again against the now-current local lake.
    #[test]
    fn prop_s4_idempotence(
        lake in gen_lake(),
        target in any::<prop::sample::Index>(),
        mutation in mutation(),
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| TestCaseError::fail(format!("runtime build: {e}")))?;
        rt.block_on(async {
            let dir = write_lake(&lake);
            let store = Arc::new(FakeStore::new());

            // Pass 1: everything uploads (fresh store).
            let (_, first) = sync(dir.path(), &store).await?;
            prop_assert!(first.failed.is_empty());
            let puts_after_first = store.puts();
            prop_assert_eq!(puts_after_first as usize, lake.files.len());

            // Pass 2, untouched lake: ZERO puts — idempotence is a
            // property of the architecture (key derivation + compare),
            // not of a flag.
            let (_, second) = sync(dir.path(), &store).await?;
            prop_assert_eq!(
                store.puts(), puts_after_first,
                "second sync must perform zero puts (uploaded {:?})", second.uploaded
            );
            prop_assert_eq!(second.uploaded.len(), 0);
            prop_assert_eq!(second.skipped.len(), lake.files.len());

            // Mutate ONE file (append / same-size edit / new file)…
            let log_before = store.put_log().len();
            let changed_rel = apply_mutation(dir.path(), &lake, &target, &mutation);

            // …pass 3: exactly the changed key re-uploads, nothing else.
            let (_, third) = sync(dir.path(), &store).await?;
            prop_assert!(third.failed.is_empty());
            let changed_keys: BTreeSet<String> =
                store.put_log()[log_before..].iter().cloned().collect();
            let expected: BTreeSet<String> =
                BTreeSet::from([format!("raw/{changed_rel}")]);
            prop_assert_eq!(
                changed_keys, expected,
                "mutation arm {:?} must re-upload exactly the changed set", mutation
            );

            // And the conservation equality holds AGAIN, against disk as
            // it is NOW (put-replaces-key is what makes this true).
            assert_conservation(&disk_files(dir.path()), &store)?;
            Ok(())
        })?;
    }
}

//! The pure core: decide what to upload, touch no network (S1, S2).
//!
//! [`plan`] takes the three inputs a decision needs — the lake root, the
//! remote prefix, and a listing somebody else already fetched — and
//! returns a [`Plan`]. Handing the listing IN (rather than calling
//! `store.list()` here) is what keeps this module pure: the properties
//! feed it a fake's listing, `--dry-run` feeds it an empty one, and the
//! real path feeds it S3's. Decisions and effects never share a function.
//!
//! ## The key rule (S1, normative)
//!
//! `key = <prefix> + <path relative to the lake root>`, forward slashes.
//! MECHANICAL on purpose — nothing in the walker's output is special-
//! cased: `dt=2026-07-09/events.jsonl` → `raw/dt=2026-07-09/events.jsonl`,
//! the real lake's `traces/dt=…/run.jsonl` → `raw/traces/dt=…/run.jsonl`,
//! and `dt=bad-ts/…` flows through verbatim (relay's quarantine partition
//! is legal lake content and syncs like any other). Nothing is ever
//! written outside the prefix.
//!
//! ## The compare (T5)
//!
//! Size first — it's already in hand from metadata, and a size mismatch
//! settles it for free. Etag second, only when sizes tie: md5 the local
//! bytes (streamed, see [`md5_of_file`]) and compare against the remote
//! etag — which IS the body md5 for single-part PUTs under SSE-S3, the
//! only kind this project writes (the caveat paragraph lives in
//! lake-store's s3.rs). The etag branch is what catches a same-size edit;
//! the S4 property forces it with an equal-length line replacement, so it
//! can never quietly become dead code behind the size check.

use std::collections::HashMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};

use lake_store::ObjectMeta;

use crate::SyncError;

/// One file the plan wants uploaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upload {
    /// Destination key (`raw/…`).
    pub key: String,
    /// Local source path.
    pub path: PathBuf,
    /// Local size in bytes (for reporting; run re-reads the body anyway).
    pub size: u64,
}

/// The decision, whole: what goes up, what stays home. Both lists are in
/// walker order (sorted), so plans are deterministic and diffable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    /// Files whose key is absent remotely or whose size/etag differs.
    pub upload: Vec<Upload>,
    /// Keys already up-to-date remotely (size AND etag match).
    pub skip: Vec<String>,
}

/// Walk `root`, compare against `remote`, decide. Pure but not
/// hermetic — it reads local metadata always, and local *bytes* only for
/// size-tied files (the etag branch).
pub fn plan(root: &Path, prefix: &str, remote: &[ObjectMeta]) -> Result<Plan, SyncError> {
    let prefix = normalized(prefix);
    // key → (size, etag) for O(1) compares; the Vec form exists because
    // the seam returns listings, not maps (S3 speaks in pages of lists).
    let by_key: HashMap<&str, (u64, &str)> = remote
        .iter()
        .map(|m| (m.key.as_str(), (m.size, m.etag.as_str())))
        .collect();

    let mut plan = Plan::default();
    // ONE String reused across every file the etag branch reads — the
    // T4 buffer-reuse discipline (see md5_of_file for what "reuse" buys).
    let mut line_buffer = String::new();

    for path in glake::walk::jsonl_files(root)? {
        let key = format!("{prefix}{}", relative_key(root, &path)?);
        let size = std::fs::metadata(&path)
            .map_err(|e| SyncError::Io {
                path: path.clone(),
                source: e,
            })?
            .len();

        let up_to_date = match by_key.get(key.as_str()) {
            None => false,                                           // not there: upload
            Some((remote_size, _)) if *remote_size != size => false, // cheap branch
            Some((_, remote_etag)) => {
                // sizes tie — the etag branch earns its keep
                md5_of_file(&path, &mut line_buffer)? == *remote_etag
            }
        };

        if up_to_date {
            plan.skip.push(key);
        } else {
            plan.upload.push(Upload { key, path, size });
        }
    }
    Ok(plan)
}

/// The S1 rule's mechanical half: `path` relative to `root`, joined with
/// forward slashes. When root IS the file (glake walks a single file
/// as-is — 003 R3b), the key is just its file name.
fn relative_key(root: &Path, path: &Path) -> Result<String, SyncError> {
    let relative = match path.strip_prefix(root) {
        // Stripping a path from ITSELF succeeds with an empty remainder —
        // the caught-by-test edge of the single-file root: the key must be
        // the file's name, never a bare prefix with nothing after it.
        Ok(rel) if !rel.as_os_str().is_empty() => rel,
        Ok(_) => Path::new(path.file_name().unwrap_or(path.as_os_str())),
        Err(_) => path,
    };
    let mut parts = Vec::new();
    for component in relative.components() {
        // Refuse non-UTF-8 rather than upload a lossily-renamed object;
        // real lake paths are ASCII (`dt=YYYY-MM-DD/…`).
        let part = component
            .as_os_str()
            .to_str()
            .ok_or_else(|| SyncError::NonUtf8Path {
                path: path.to_path_buf(),
            })?;
        parts.push(part);
    }
    // components() already normalized away `.` and separators; joining
    // with '/' makes the key OS-independent (Path would use '\' on
    // Windows — S3 keys never do).
    Ok(parts.join("/"))
}

/// A prefix always ends in exactly one `/` so key concatenation can be
/// dumb: `raw` and `raw/` both mean `raw/…`.
fn normalized(prefix: &str) -> String {
    format!("{}/", prefix.trim_end_matches('/'))
}

/// md5 of a file's bytes, read line by line through ONE caller-owned,
/// reused buffer (T4).
///
/// Honesty box about "streaming": the DIGEST streams — each line goes
/// through `Context::consume` and is then overwritten by the next, so
/// peak memory is one line, not one file. Nothing is accumulated here
/// (contrast run.rs, where the PUT body must be whole). `read_line`
/// keeps every byte it read — the `\n`s included, a final line without
/// one included — so consuming the buffers in order digests EXACTLY the
/// file's bytes; that exactness is what makes the etag compare mean
/// anything. `clear()` resets length, not capacity: after the first long
/// line the buffer stops allocating — that is the whole reuse lesson.
/// (Cost of the discipline: read_line validates UTF-8, which byte-chunk
/// reading wouldn't; the lake is UTF-8 JSONL by construction, and a
/// non-UTF-8 file surfaces as an io error here instead of syncing.)
fn md5_of_file(path: &Path, line_buffer: &mut String) -> Result<String, SyncError> {
    let file = std::fs::File::open(path).map_err(|e| SyncError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let mut reader = std::io::BufReader::new(file);
    let mut context = md5::Context::new();
    loop {
        line_buffer.clear();
        let n = reader.read_line(line_buffer).map_err(|e| SyncError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        if n == 0 {
            break; // true EOF (read_line returns Ok(0) only there)
        }
        context.consume(line_buffer.as_bytes());
    }
    Ok(format!("{:x}", context.finalize()))
}

/// Read one whole file body through the same reused-line-buffer
/// discipline — run.rs uses this to build PUT bodies. Here the honesty
/// box tips the other way: the LINES stream through the reused buffer,
/// but the BODY accumulates — a single PUT needs the whole object, so
/// peak memory is one file (KB-scale by design; multipart is a non-goal).
pub(crate) fn read_body(path: &Path, line_buffer: &mut String) -> Result<Vec<u8>, SyncError> {
    let file = std::fs::File::open(path).map_err(|e| SyncError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let mut reader = std::io::BufReader::new(file);
    let mut body = Vec::new();
    loop {
        line_buffer.clear();
        let n = reader.read_line(line_buffer).map_err(|e| SyncError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(line_buffer.as_bytes());
    }
    Ok(body)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Build a little lake on disk: (relative path, content) pairs.
    fn lake(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (rel, content) in files {
            let path = dir.path().join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        dir
    }

    fn meta(key: &str, size: u64, etag: &str) -> ObjectMeta {
        ObjectMeta {
            key: key.to_owned(),
            size,
            etag: etag.to_owned(),
        }
    }

    /// [E] S1 — the key rule, normative table: `raw/` + path relative to
    /// root; `traces/` and `dt=bad-ts` flow through VERBATIM.
    #[test]
    fn s1_keys_are_prefix_plus_relative_path_verbatim() {
        let dir = lake(&[
            ("dt=2026-07-05/events.jsonl", "{}\n"),
            (
                "dt=bad-ts/events.jsonl",
                "quarantine is legal lake content\n",
            ),
            ("traces/dt=2026-07-05/run-1.jsonl", "{\"t\":1}\n"),
        ]);
        let plan = plan(dir.path(), "raw/", &[]).unwrap();
        let keys: Vec<&str> = plan.upload.iter().map(|u| u.key.as_str()).collect();
        assert_eq!(
            keys,
            [
                "raw/dt=2026-07-05/events.jsonl",
                "raw/dt=bad-ts/events.jsonl",
                "raw/traces/dt=2026-07-05/run-1.jsonl",
            ],
            "sorted walker order, nothing special-cased"
        );
        assert!(plan.skip.is_empty());
        assert!(
            keys.iter().all(|k| k.starts_with("raw/")),
            "never outside raw/"
        );
    }

    /// Root-is-a-file (003 R3b) and prefix normalization (`raw` == `raw/`).
    #[test]
    fn s1_single_file_root_and_prefix_normalization() {
        let dir = lake(&[("events.jsonl", "x\n")]);
        let file = dir.path().join("events.jsonl");
        let plan = plan(&file, "raw", &[]).unwrap();
        assert_eq!(plan.upload.len(), 1);
        assert_eq!(plan.upload[0].key, "raw/events.jsonl");
    }

    /// The compare, all three verdicts: absent → upload; size differs →
    /// upload (etag never read); size ties + etag differs → upload; both
    /// tie → skip.
    #[test]
    fn t5_size_first_etag_second() {
        let dir = lake(&[
            ("dt=a/events.jsonl", "one\n"), // absent remotely
            ("dt=b/events.jsonl", "two\n"), // remote size differs
            ("dt=c/events.jsonl", "tre\n"), // same size, different bytes
            ("dt=d/events.jsonl", "for\n"), // identical
        ]);
        let remote = [
            meta("raw/dt=b/events.jsonl", 999, "irrelevant — size settles it"),
            meta(
                "raw/dt=c/events.jsonl",
                4,
                &format!("{:x}", md5::compute(b"xxx\n")),
            ),
            meta(
                "raw/dt=d/events.jsonl",
                4,
                &format!("{:x}", md5::compute(b"for\n")),
            ),
        ];
        let plan = plan(dir.path(), "raw/", &remote).unwrap();
        let uploads: Vec<&str> = plan.upload.iter().map(|u| u.key.as_str()).collect();
        assert_eq!(
            uploads,
            [
                "raw/dt=a/events.jsonl",
                "raw/dt=b/events.jsonl",
                "raw/dt=c/events.jsonl"
            ]
        );
        assert_eq!(plan.skip, ["raw/dt=d/events.jsonl"]);
    }

    /// The walker's error discipline passes through: missing root → Err
    /// with path context (main turns this into stderr + exit 2, S5).
    #[test]
    fn s5_missing_root_is_an_error_with_path_context() {
        let e = plan(Path::new("/no/such/lake"), "raw/", &[]).unwrap_err();
        assert!(e.to_string().contains("/no/such/lake"), "{e}");
    }

    /// read_body reproduces a file's bytes EXACTLY — trailing-newline-less
    /// last lines included (etag correctness depends on this).
    #[test]
    fn t4_read_body_is_byte_exact() {
        let dir = lake(&[("dt=a/events.jsonl", "line1\n\nline3")]); // blank line, no trailing \n
        let mut buffer = String::new();
        let body = read_body(&dir.path().join("dt=a/events.jsonl"), &mut buffer).unwrap();
        assert_eq!(body, b"line1\n\nline3");
        assert_eq!(
            format!("{:x}", md5::compute(&body)),
            md5_of_file(&dir.path().join("dt=a/events.jsonl"), &mut buffer).unwrap(),
            "digest path and body path agree byte-for-byte"
        );
    }
}

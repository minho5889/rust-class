//! Sitting H: one designed error type instead of ad-hoc `eprintln!`s.
//!
//! The v0 binary matched on `io::Result` at every call site and printed
//! whatever it had. v1 gives the library ONE public error enum (F5): every
//! fallible library path returns `Result<_, GlakeError>`, and `main` maps
//! any error to a single stderr line (`glake: <Display>`) plus exit code 2.
//!
//! `thiserror` writes the boring parts (the `Display` impl, the
//! `std::error::Error` impl, the `source()` chain) from attributes — but the
//! *design* is still ours: which variants exist, what context each carries,
//! and what the user-facing message says.
//!
//! C-GOOD-ERR rubric (F6), checked by the tests below:
//! - messages are lowercase and have no trailing period;
//! - the type is `std::error::Error + Send + Sync + 'static`;
//! - the underlying cause is reachable through `source()`, not smashed
//!   into the message-and-nothing-else.
//!
//! Note on `#[from]` (F5 as amended): a bare `#[from] io::Error` would let
//! `?` convert automatically, but it can't work here — `Io` carries the
//! *path* as context, and an `io::Error` alone doesn't know the path. So we
//! use `#[source]` and attach the path explicitly with the `GlakeError::io`
//! helper at each call site (`.map_err(|e| GlakeError::io(path, e))`).
//! Context costs one closure; anonymous errors cost a debugging session.

use std::path::{Path, PathBuf};

/// Everything that can go wrong inside the glake library.
///
/// Two variants are enough at v1 — resist inventing error taxonomy before
/// there are callers who would match on it.
#[derive(Debug, thiserror::Error)]
pub enum GlakeError {
    /// A filesystem operation failed. Carries the path we were touching
    /// (the context the user needs) and the original `io::Error` as
    /// `source()` (the cause a tool or a `{:#}`-style printer can walk).
    ///
    /// Renders as e.g. `cannot read /root/forbidden: permission denied
    /// (os error 13)` — lowercase, no trailing period (C-GOOD-ERR).
    #[error("cannot read {}: {source}", path.display())]
    Io {
        /// What we were trying to read when it failed.
        path: PathBuf,
        /// The underlying io error, preserved as the chained `source()`.
        #[source]
        source: std::io::Error,
    },

    /// The arguments parsed (so clap was happy) but their *values* make no
    /// sense — e.g. `--since 2026/07/06`. Maps to exit code 2 like every
    /// other usage problem.
    #[error("{0}")]
    Usage(String),
}

impl GlakeError {
    /// Attach path context to an `io::Error` — the `map_err` companion:
    /// `std::fs::read_to_string(p).map_err(|e| GlakeError::io(p, e))`.
    pub fn io(path: &Path, source: std::io::Error) -> Self {
        GlakeError::Io {
            path: path.to_path_buf(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    /// F6: the error type must be usable across threads and boxable as
    /// `Box<dyn Error + Send + Sync>` — checked at compile time.
    #[test]
    fn f6_error_is_send_sync_static() {
        fn assert_good_err<T: std::error::Error + Send + Sync + 'static>() {}
        assert_good_err::<GlakeError>();
    }

    /// F6: display style — lowercase start, no trailing period, and the io
    /// variant leads with the path context.
    #[test]
    fn f6_display_is_lowercase_no_period_with_context() {
        let io = GlakeError::io(
            Path::new("/root/forbidden"),
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied"),
        );
        let usage = GlakeError::Usage("--since expects YYYY-MM-DD, got \"nope\"".into());
        for e in [&io, &usage] {
            let msg = e.to_string();
            assert!(
                !msg.starts_with(char::is_uppercase),
                "starts lowercase: {msg}"
            );
            assert!(!msg.ends_with('.'), "no trailing period: {msg}");
        }
        assert!(io.to_string().starts_with("cannot read /root/forbidden:"));
    }

    /// F6: the io cause is chained through `source()`, not lost.
    #[test]
    fn f6_source_chain_reaches_the_io_error() {
        let e = GlakeError::io(
            Path::new("/x"),
            std::io::Error::new(std::io::ErrorKind::NotFound, "gone"),
        );
        let source = e.source().expect("Io chains its source");
        assert!(source.to_string().contains("gone"));
        assert!(GlakeError::Usage("bad".into()).source().is_none());
    }
}

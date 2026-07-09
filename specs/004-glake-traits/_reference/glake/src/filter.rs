//! Sitting I: filters as data plus a closure-shaped predicate — the T5
//! lesson (closures and iterator adapters doing real work).
//!
//! A [`Filter`] is just the two optional constraints from the CLI. It
//! doesn't iterate anything itself; the pipeline applies it with ordinary
//! iterator adapters (see `tally::tally_filtered`). Filters are
//! **stats-only** (F1 as amended): `validate` doesn't accept them, because
//! malformed lines have no `event_type` or day to filter on.
//!
//! Design note (amendment 7): a plain `keep() -> bool` can't tell the
//! caller *why* an event was dropped, and F2 needs one particular why —
//! "excluded because `--since` can't compare a bad-ts day" — surfaced as a
//! count. So the primary API is [`Filter::verdict`], a three-way answer;
//! [`Filter::keep`] stays as the boolean convenience for callers (and
//! properties) that only care about the partition.

use crate::classify::is_day;
use crate::error::GlakeError;
use crate::parser::ClassifiedLine;

/// The `--type` / `--since` constraints, as plain data (F1, F2).
/// `Default` is the no-op filter: everything kept.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    /// `--type <t>`: keep only events whose `event_type` equals this
    /// EXACTLY (no globs, no prefixes — F1).
    pub kind: Option<String>,
    /// `--since <YYYY-MM-DD>`: keep only events whose day is on/after this
    /// date. Days and dates share one shape (the F2 day rule), so plain
    /// `&str >=` IS chronological order — no date library needed.
    pub since: Option<String>,
}

/// What [`Filter::verdict`] says about one classified line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Passes every active constraint (non-events always do — filters
    /// constrain *events*; blank/malformed/unparseable lines flow through
    /// so the malformed count is never silently filtered away).
    Keep,
    /// An event that fails `--type`, or whose day is before `--since`.
    Skip,
    /// An event excluded ONLY because `--since` is active and the event's
    /// day is the `bad-ts` bucket — there is no day to compare (F2).
    /// Counted separately so stats can print the one-line note.
    SkipBadTs,
}

impl Filter {
    /// Build a filter from raw CLI values, validating that `--since` is
    /// `YYYY-MM-DD`-shaped (same [`is_day`] rule the classifiers use) —
    /// a nonsense date would silently compare wrong, so it's a usage error
    /// (exit 2) instead.
    pub fn new(kind: Option<String>, since: Option<String>) -> Result<Self, GlakeError> {
        if let Some(s) = &since
            && !is_day(s)
        {
            return Err(GlakeError::Usage(format!(
                "--since expects YYYY-MM-DD, got {s:?}"
            )));
        }
        Ok(Filter { kind, since })
    }

    /// Is any constraint set? Stats prints `(filtered from M)` only when
    /// this is true (F4) — an unfiltered run keeps v0's exact output.
    pub fn is_active(&self) -> bool {
        self.kind.is_some() || self.since.is_some()
    }

    /// The three-way answer for one line. Checks `--type` before
    /// `--since`: an event of the wrong type is a plain [`Verdict::Skip`]
    /// even if its day is also bad-ts — [`Verdict::SkipBadTs`] is reserved
    /// for events the since-rule alone excluded, so the printed note counts
    /// exactly what F2 describes.
    pub fn verdict(&self, line: &ClassifiedLine) -> Verdict {
        // Non-events flow through untouched (see Verdict::Keep docs).
        let ClassifiedLine::Event { kind, day } = line else {
            return Verdict::Keep;
        };
        // Option combinators as the predicate: None = constraint not set.
        if self.kind.as_deref().is_some_and(|want| want != kind) {
            return Verdict::Skip;
        }
        match self.since.as_deref() {
            None => Verdict::Keep,
            Some(_) if day == "bad-ts" => Verdict::SkipBadTs,
            Some(since) => {
                if day.as_str() >= since {
                    Verdict::Keep
                } else {
                    Verdict::Skip
                }
            }
        }
    }

    /// The boolean view of [`Filter::verdict`] — the partition predicate
    /// the F3 property is stated over.
    pub fn keep(&self, line: &ClassifiedLine) -> bool {
        matches!(self.verdict(line), Verdict::Keep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: &str, day: &str) -> ClassifiedLine {
        ClassifiedLine::Event {
            kind: kind.to_owned(),
            day: day.to_owned(),
        }
    }

    /// [E] F1: exact type match only.
    #[test]
    fn f1_type_filter_is_exact() {
        let f = Filter {
            kind: Some("gate.approved".into()),
            since: None,
        };
        assert!(f.keep(&event("gate.approved", "2026-07-09")));
        assert!(!f.keep(&event("gate.approved.extra", "2026-07-09")));
        assert!(!f.keep(&event("gate", "2026-07-09")));
    }

    /// [E] F2: day >= date kept; bad-ts is its own verdict.
    #[test]
    fn f2_since_keeps_on_or_after_and_flags_bad_ts() {
        let f = Filter {
            kind: None,
            since: Some("2026-07-06".into()),
        };
        assert_eq!(f.verdict(&event("a.b", "2026-07-06")), Verdict::Keep);
        assert_eq!(f.verdict(&event("a.b", "2026-07-09")), Verdict::Keep);
        assert_eq!(f.verdict(&event("a.b", "2026-07-05")), Verdict::Skip);
        assert_eq!(f.verdict(&event("a.b", "bad-ts")), Verdict::SkipBadTs);
        // ...but without --since, bad-ts events are ordinary keeps:
        assert_eq!(
            Filter::default().verdict(&event("a.b", "bad-ts")),
            Verdict::Keep
        );
    }

    /// Type check wins over since: wrong-type bad-ts events are Skip, not
    /// SkipBadTs (the note counts only what the since-rule excluded).
    #[test]
    fn f2_wrong_type_bad_ts_is_plain_skip() {
        let f = Filter {
            kind: Some("gate.approved".into()),
            since: Some("2026-07-06".into()),
        };
        assert_eq!(f.verdict(&event("other.kind", "bad-ts")), Verdict::Skip);
        assert_eq!(
            f.verdict(&event("gate.approved", "bad-ts")),
            Verdict::SkipBadTs
        );
    }

    /// Non-events always flow through, whatever the filter.
    #[test]
    fn non_events_always_keep() {
        let f = Filter {
            kind: Some("x.y".into()),
            since: Some("2026-07-06".into()),
        };
        assert!(f.keep(&ClassifiedLine::Blank));
        assert!(f.keep(&ClassifiedLine::Unparseable));
        assert!(f.keep(&ClassifiedLine::Malformed {
            missing: "actor".into()
        }));
    }

    /// Constructor validates the --since shape (usage error, F5/F10).
    #[test]
    fn new_rejects_non_day_since() {
        assert!(Filter::new(None, Some("2026-07-06".into())).is_ok());
        for bad in ["2026/07/06", "yesterday", "2026-7-6", ""] {
            let err = Filter::new(None, Some(bad.into())).expect_err(bad);
            assert!(matches!(err, GlakeError::Usage(_)), "{bad} → usage error");
        }
    }
}

//! Sitting I: the hand-rolled `std::env::args` matcher retires; `clap`
//! derive takes over (F10). The struct below IS the interface — clap
//! generates parsing, `--help`, and error handling from it, and its
//! default behavior already matches v0's contract: bad usage prints to
//! stderr and **exits 2** (verified by tests/cli.rs).
//!
//! Filters are stats-only (F1 as amended): `--type`/`--since` live on the
//! `stats` subcommand ONLY, so `glake validate lake --type x` is rejected
//! by clap itself ("unexpected argument", exit 2) — malformed lines have
//! no event_type or day to filter on. `--parser` is NOT a filter and both
//! commands take it.

use crate::parser::{EventParser, HandParser, SerdeParser};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// glake — the goldeneye lake CLI (spec 004 reference, v1).
#[derive(Debug, Parser)]
#[command(
    name = "glake",
    version,
    about = "inspect the goldeneye telemetry lake"
)]
pub struct Cli {
    /// Which job to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The two v0 commands — no new ones in v1 (a spec non-goal).
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Report lines that violate the envelope schema
    Validate {
        /// A .jsonl file, or a directory to scan recursively
        path: PathBuf,
        /// Parsing backend to classify lines with
        #[arg(long, value_enum, default_value_t)]
        parser: ParserChoice,
    },
    /// Count events by type and by day
    Stats {
        /// A .jsonl file, or a directory to scan recursively
        path: PathBuf,
        /// Keep only events whose event_type equals this exactly
        #[arg(long = "type", value_name = "EVENT_TYPE")]
        kind: Option<String>,
        /// Keep only events whose day is on or after this date
        #[arg(long, value_name = "YYYY-MM-DD")]
        since: Option<String>,
        /// Parsing backend to classify lines with
        #[arg(long, value_enum, default_value_t)]
        parser: ParserChoice,
    },
}

/// The `--parser` flag (F7): which [`EventParser`] backend classifies
/// lines. clap's `ValueEnum` derive turns the variant names into the
/// accepted spellings `hand` and `serde`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum ParserChoice {
    /// The 003 hand-rolled scanner (zero-copy core; the default)
    #[default]
    Hand,
    /// serde_json parsing to a Value (strict; the rival on trial)
    Serde,
}

impl ParserChoice {
    /// The runtime seam (F7/F13): turn the flag into a trait object. This
    /// `Box<dyn EventParser>` is the ONLY dynamic-dispatch point in the
    /// crate — everything downstream is generic and would monomorphize if
    /// handed a concrete parser instead.
    pub fn build(self) -> Box<dyn EventParser> {
        match self {
            ParserChoice::Hand => Box::new(HandParser),
            ParserChoice::Serde => Box::new(SerdeParser),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// clap's own debug-assert pass over the derive: catches conflicting
    /// flags, bad defaults, etc. at test time.
    #[test]
    fn f10_clap_definition_is_coherent() {
        use clap::CommandFactory as _;
        Cli::command().debug_assert();
    }

    /// The flag spellings are exactly hand|serde, defaulting to hand (F7).
    #[test]
    fn f7_parser_choice_spellings() {
        assert_eq!(ParserChoice::default(), ParserChoice::Hand);
        let cli = Cli::try_parse_from(["glake", "stats", "lake", "--parser", "serde"])
            .expect("serde spelling parses");
        let Command::Stats { parser, .. } = cli.command else {
            panic!("expected stats");
        };
        assert_eq!(parser, ParserChoice::Serde);
        assert!(Cli::try_parse_from(["glake", "stats", "lake", "--parser", "simd"]).is_err());
    }

    /// Filters are stats-only: validate rejects --type/--since at the
    /// definition level (F1 as amended).
    #[test]
    fn f1_validate_has_no_filter_flags() {
        assert!(Cli::try_parse_from(["glake", "validate", "lake", "--type", "x"]).is_err());
        assert!(
            Cli::try_parse_from(["glake", "validate", "lake", "--since", "2026-07-06"]).is_err()
        );
        assert!(Cli::try_parse_from(["glake", "validate", "lake", "--parser", "serde"]).is_ok());
    }
}

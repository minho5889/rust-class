//! The binary: clap parses, this boundary owns the exit codes (glake's
//! discipline, normative here too — S2/S5):
//!
//! - `2` — usage problems (clap's own default) and io/list failures
//!   (missing path, unreadable file, LIST refused);
//! - `1` — the sync ran but one or more puts failed (partial progress
//!   already reported);
//! - `0` — clean (including "nothing to do").
//!
//! ## Why `--dry-run` never touches AWS (S2, structural)
//!
//! The dry-run path builds NO client, loads NO credentials, and plans
//! against an EMPTY remote listing — it answers "what does the walker see
//! and where would it go?", which needs no network and therefore works on
//! a machine with no AWS account at all (this reference was authored on
//! one). The honest consequence: a dry-run plan is the FIRST-sync plan —
//! an upper bound that can't know what an already-populated bucket would
//! skip. `--bucket` may still be passed (the transcript you'll actually
//! type on deploy day keeps it); it is simply not consulted.
//!
//! The real path is the only impure dozen lines in the crate: load the
//! default credential/region chain (env → profile → IMDS, the same chain
//! every AWS tool speaks), build the client ONCE, LIST once, then hand
//! everything to the pure core.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use lake_store::ObjectStore;
use lake_store::s3::{S3Client, S3Store};
use lake_sync::plan::{Plan, plan};
use lake_sync::{SyncError, run};

/// lake-sync — push the local goldeneye lake to S3, idempotently.
#[derive(Debug, Parser)]
#[command(
    name = "lake-sync",
    version,
    about = "sync the local lake to s3://<bucket>/<prefix>"
)]
struct Cli {
    /// Destination bucket (needed unless --dry-run)
    #[arg(long, required_unless_present = "dry_run")]
    bucket: Option<String>,

    /// Plan only: print what would upload/skip, perform zero puts
    #[arg(long)]
    dry_run: bool,

    /// Remote key prefix every object lands under
    #[arg(long, default_value = "raw/")]
    prefix: String,

    /// The local lake root (a directory, or a single .jsonl file)
    path: PathBuf,
}

#[tokio::main]
async fn main() -> ExitCode {
    // clap owns bad usage: prints its message + usage to stderr, exits 2.
    let cli = Cli::parse();

    // One place turns any "could not even try" error into the user line.
    match run_cli(cli).await {
        Ok(code) => code,
        Err(e) => {
            eprintln!("lake-sync: {e}");
            ExitCode::from(2)
        }
    }
}

async fn run_cli(cli: Cli) -> Result<ExitCode, SyncError> {
    if cli.dry_run {
        // Plan against an empty remote — see the module docs for why.
        let plan = plan(&cli.path, &cli.prefix, &[])?;
        print!("{}", render_plan(&plan));
        return Ok(ExitCode::SUCCESS);
    }

    // clap's required_unless_present guarantees Some here; the match is
    // the no-unwrap spelling of that guarantee (defense costs two lines).
    let Some(bucket) = cli.bucket else {
        return Err(SyncError::Io {
            path: cli.path,
            source: std::io::Error::other("--bucket is required without --dry-run"),
        });
    };

    // The client, built ONCE before any work — the same client-once
    // discipline hello-lambda's main teaches for cold starts; here it
    // just means the credential chain resolves once, not per file.
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let store = Arc::new(S3Store::new(S3Client::new(&config), bucket));

    // LIST once, plan pure, run bounded.
    let remote = store.list(&cli.prefix).await?;
    let plan = plan(&cli.path, &cli.prefix, &remote)?;
    let outcome = run::run(store, plan).await?;

    print!("{}", outcome.stdout_report());
    eprint!("{}", outcome.stderr_report());
    Ok(ExitCode::from(outcome.exit_code()))
}

/// The dry-run transcript: every planned upload named, then the summary
/// (`plan: upload N, skip M` — the requirements shape, minus the "(NNN
/// events)" decoration: counting events is glake's job, and the plan
/// deliberately never parses lake content, it moves bytes).
fn render_plan(plan: &Plan) -> String {
    let mut out = String::new();
    for upload in &plan.upload {
        out.push_str(&format!("upload {} ({} bytes)\n", upload.key, upload.size));
    }
    for key in &plan.skip {
        out.push_str(&format!("skip {key}\n"));
    }
    out.push_str(&format!(
        "plan: upload {}, skip {}\n",
        plan.upload.len(),
        plan.skip.len()
    ));
    out
}

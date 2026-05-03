use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use crate::{
    PerlcheckerError, Result, V1_LANGUAGE_SUBSET, extractor,
    symexec::{Counterexample, ModelValue, VerificationResult, verify_extracted_functions},
};

#[derive(Debug, Parser)]
#[command(
    name = "perlchecker",
    version,
    about = "Symbolic verification for a strict Perl subset"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Check {
        /// Perl source file to analyze.
        path: PathBuf,
    },
}

pub fn run() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();

    match cli.command {
        Commands::Check { path } => run_check(path),
    }
}

fn run_check(path: PathBuf) -> Result<()> {
    info!(
        supported_types = ?V1_LANGUAGE_SUBSET.supported_types,
        "running full verification pipeline"
    );

    let source = fs::read_to_string(&path).map_err(|source| PerlcheckerError::ReadFile {
        path: path.clone(),
        source,
    })?;
    let functions = extractor::extract_annotated_functions(&source)?;

    debug!(function_count = functions.len(), "extraction completed");
    if functions.is_empty() {
        println!("Found 0 annotated functions");
        return Ok(());
    }

    let mut failed = false;
    for result in verify_extracted_functions(&functions)? {
        match result {
            VerificationResult::Verified { function } => {
                println!("✔ {function}: verified");
            }
            VerificationResult::Counterexample(example) => {
                failed = true;
                print_counterexample(&example);
            }
        }
    }

    if failed {
        Err(PerlcheckerError::VerificationFailed)
    } else {
        Ok(())
    }
}

fn print_counterexample(example: &Counterexample) {
    println!("✘ {}: counterexample found", example.function);
    println!("Function {} failed:", example.function);
    for (name, value) in &example.assignments {
        match value {
            ModelValue::Int(value) => println!("  {name} = {value}"),
            ModelValue::Str(value) => println!("  {name} = \"{value}\""),
            ModelValue::Collection(value) => println!("  {name} = {value}"),
        }
    }
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_target(false)
        .without_time()
        .try_init();
}

use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use crate::{
    PerlcheckerError, Result, V1_LANGUAGE_SUBSET, extractor,
    limits::{DEFAULT_MAX_LOOP_UNROLL, DEFAULT_MAX_PATHS, DEFAULT_SOLVER_TIMEOUT_MS, Limits},
    symexec::{Counterexample, ModelValue, VerificationResult, verify_extracted_functions_with_externs},
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

        /// Maximum loop unroll depth.
        #[arg(long, default_value_t = DEFAULT_MAX_LOOP_UNROLL)]
        max_loop_unroll: usize,

        /// Maximum number of symbolic execution paths.
        #[arg(long, default_value_t = DEFAULT_MAX_PATHS)]
        max_paths: usize,

        /// SMT solver timeout in milliseconds.
        #[arg(long, default_value_t = DEFAULT_SOLVER_TIMEOUT_MS)]
        solver_timeout_ms: u32,
    },
}

pub fn run() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();

    match cli.command {
        Commands::Check {
            path,
            max_loop_unroll,
            max_paths,
            solver_timeout_ms,
        } => run_check(
            path,
            Limits {
                max_loop_unroll,
                max_paths,
                solver_timeout_ms,
            },
        ),
    }
}

fn run_check(path: PathBuf, limits: Limits) -> Result<()> {
    info!(
        supported_types = ?V1_LANGUAGE_SUBSET.supported_types,
        "running full verification pipeline"
    );

    let source = fs::read_to_string(&path).map_err(|source| PerlcheckerError::ReadFile {
        path: path.clone(),
        source,
    })?;
    let functions = extractor::extract_annotated_functions(&source)?;
    let extern_lines = extractor::extract_extern_lines(&source);

    debug!(
        function_count = functions.len(),
        extern_count = extern_lines.len(),
        "extraction completed"
    );
    if functions.is_empty() {
        println!("Found 0 annotated functions");
        return Ok(());
    }

    let mut failed = false;
    for result in verify_extracted_functions_with_externs(&functions, &extern_lines, limits)? {
        match result {
            VerificationResult::Verified { function } => {
                println!("✔ {function}: verified");
            }
            VerificationResult::Counterexample(example) => {
                failed = true;
                print_counterexample(&example);
            }
            VerificationResult::Unknown { function, max_unroll } => {
                failed = true;
                print_unknown(&function, max_unroll);
            }
        }
    }

    if failed {
        Err(PerlcheckerError::VerificationFailed)
    } else {
        Ok(())
    }
}

fn print_unknown(function: &str, max_unroll: usize) {
    println!(
        "\u{26a0} {function}: UNKNOWN \u{2014} loop may exceed unroll bound (max {max_unroll} iterations)"
    );
    println!("  Add a loop invariant: # inv: <your invariant expression>");
    println!("  The invariant must hold on loop entry, be preserved by each iteration,");
    println!("  and together with the exit condition imply the postcondition.");
}

fn print_counterexample(example: &Counterexample) {
    println!("✘ {}: counterexample found", example.function);
    println!("Function {} failed:", example.function);
    for (name, value) in &example.assignments {
        match value {
            ModelValue::Int(value) => println!("  {name} = {value}"),
            ModelValue::Str(value) => println!("  {name} = \"{value}\""),
            ModelValue::Collection(value) => println!("  {name} = {value}"),
            ModelValue::Unconstrained => println!("  {name} = <unconstrained>"),
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

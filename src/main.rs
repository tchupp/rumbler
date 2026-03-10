mod apply;
mod config;
mod db;
mod error;
mod lint;
mod migration;
mod reverse;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(
    name = "rumbler",
    about = "A simple SQL schema migration tool for PostgreSQL"
)]
struct Cli {
    #[arg(short = 'c', long = "configuration")]
    configuration: Option<String>,

    #[arg(short = 'e', long = "environment")]
    environment: Option<String>,

    #[arg(long)]
    debug: bool,

    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Apply {
        #[arg(short, long, conflicts_with = "migration")]
        all: bool,

        #[arg(long)]
        no_save: bool,

        #[arg(long, conflicts_with = "all")]
        migration: Option<String>,
    },
    Reverse {
        #[arg(short, long, conflicts_with = "migration")]
        all: bool,

        #[arg(long)]
        no_save: bool,

        #[arg(long, conflicts_with = "all")]
        migration: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let log_level = if cli.debug {
        "debug,sqruff_lib=warn,sqruff_lib_core=warn,sqruff_lib_dialects=warn"
    } else {
        "info,sqruff_lib=warn,sqruff_lib_core=warn,sqruff_lib_dialects=warn"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .format_target(false)
        .init();

    if let Err(e) = run(cli) {
        log::error!("{e}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), error::RumblerError> {
    let cfg = config::load(cli.configuration, cli.environment)?;

    match cli.command {
        Command::Apply {
            all,
            no_save,
            migration,
        } => {
            let mut db = db::Db::connect(&cfg)?;
            let options = apply::ApplyOptions {
                all,
                no_save,
                dry_run: cli.dry_run,
                migration_filter: migration,
            };
            apply::run(&mut db, &cfg.directory, &options)?;
        }
        Command::Reverse {
            all,
            no_save,
            migration,
        } => {
            let mut db = db::Db::connect(&cfg)?;
            let options = reverse::ReverseOptions {
                all,
                no_save,
                dry_run: cli.dry_run,
                migration_filter: migration,
            };
            reverse::run(&mut db, &cfg.directory, &options)?;
        }
    }

    Ok(())
}

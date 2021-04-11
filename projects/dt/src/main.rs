use clap::{App, AppSettings, Arg};
use log::LevelFilter;
use std::process::exit;

mod command;
mod validation;

fn main() {
    let matches = App::new("dt")
        .global_setting(AppSettings::HelpRequired)
        .global_setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .about("Prints verbose logs")
                .global(true)
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .short('n')
                .about("Performs a trial run with no changes made")
                .global(true),
        )
        .about("Tools to manage the duck git repository")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            App::new("cargo-test")
                .about("Run `cross test` for a Rust project")
                .arg(
                    Arg::new("project-dir")
                        .required(true)
                        .about("Path to one of Duck's Rust projects")
                        .validator(validation::validate_dir),
                )
                .arg(
                    Arg::new("windows")
                        .long("windows")
                        .short('w')
                        .about("Build for windows"),
                ),
        )
        .subcommand(
            App::new("repo-rsync")
                .about("Sync files from a Duck project to its own git repository directory")
                .arg(
                    Arg::new("project-dir")
                        .required(true)
                        .about("Path to one of Duck's Rust projects")
                        .validator(validation::validate_dir),
                )
                .arg(
                    Arg::new("repo-dir")
                        .required(true)
                        .about("Path to the git repository for that Rust project")
                        .validator(validation::validate_dir),
                ),
        )
        .subcommand(
            App::new("repo-funding")
                .about("Copies a FUNDING.yml file to a git repository directory")
                .arg(
                    Arg::new("repo-dir")
                        .required(true)
                        .about("Path to the git repository for that Rust project")
                        .validator(validation::validate_git_repo),
                )
                .arg(
                    Arg::new("funding-file")
                        .required(true)
                        .about("Funding file to copy to the given git repository")
                        .validator(validation::validate_file),
                ),
        )
        .get_matches();

    let dry_run = matches.is_present("dry-run");
    let verbose = matches.occurrences_of("verbose");
    let log_level = if verbose >= 3 {
        LevelFilter::Trace
    } else if verbose >= 2 {
        LevelFilter::Debug
    } else if verbose >= 1 {
        LevelFilter::Info
    } else {
        LevelFilter::Off
    };
    env_logger::builder().filter_level(log_level).init();

    log::debug!("{:?}", matches);

    if dry_run {
        log::info!("dry-run active");
    }

    let result = match matches.subcommand() {
        Some(("cargo-test", subcommand_matches)) => {
            command::command_cargo_test(dry_run, log_level, subcommand_matches)
        }
        Some(("repo-rsync", subcommand_matches)) => {
            command::command_repo_rsync(dry_run, log_level, subcommand_matches)
        }
        Some(("repo-funding", subcommand_matches)) => {
            command::command_repo_funding(dry_run, subcommand_matches)
        }
        _ => unimplemented!(),
    };

    match result {
        Err(msg) => {
            log::error!("{}", msg);
            exit(1);
        }
        Ok(_) => {}
    }
}

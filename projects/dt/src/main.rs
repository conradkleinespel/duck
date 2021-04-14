use std::process::exit;

use clap::{App, AppSettings, Arg};
use log::LevelFilter;

use command::{cargo_test, repo_history};

mod command;
#[allow(unused)]
mod rclio;
#[allow(unused)]
mod rpassword;
#[allow(unused)]
mod rprompt;
#[allow(unused)]
mod rutil;
mod validation;

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut io = rclio::RegularInputOutput::new(stdin.lock(), stdout.lock(), stderr.lock());

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
            App::new("repo-history")
                .about("Replay history from Duck onto a single project git repository")
                .arg(
                    Arg::new("duck-repo")
                        .required(true)
                        .about("HTTPS url to Duck's Git repository"),
                )
                .arg(
                    Arg::new("project-name-in-duck")
                        .required(true)
                        .about("The name of the project in Duck"),
                )
                .arg(
                    Arg::new("project-repo")
                        .required(true)
                        .about("HTTPS url to the single project repository"),
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
            cargo_test::command_cargo_test(dry_run, log_level, subcommand_matches)
        }
        Some(("repo-history", subcommand_matches)) => {
            repo_history::command_repo_history(&mut io, dry_run, log_level, subcommand_matches)
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

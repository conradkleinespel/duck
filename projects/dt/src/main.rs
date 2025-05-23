use std::process::exit;

use clap::{Arg, ArgAction, Command};
use log::LevelFilter;

pub mod repo_history;

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut io = rclio::RegularInputOutput::new(
        stdin.lock(),
        stdout.lock(),
        stderr.lock(),
        false,
    );

    let matches = Command::new("dt")
        .help_expected(true)
        .disable_help_subcommand(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("Prints verbose logs")
                .global(true)
                .action(ArgAction::Count),
        )
        .arg(
            Arg::new("dry-run")
                .action(ArgAction::SetTrue)
                .long("dry-run")
                .short('n')
                .help("Performs a trial run with no changes made")
                .global(true),
        )
        .about("Tools to manage the duck git repository")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("repo-history")
                .about("Replay history from Duck onto a single project git repository")
                .arg(
                    Arg::new("project-name-in-duck")
                        .required(true)
                        .help("The name of the project in Duck"),
                )
                .arg(
                    Arg::new("duck-repo")
                        .long("duck-repo")
                        .help("HTTPS url to Duck's Git repository"),
                )
                .arg(
                    Arg::new("duck-branch")
                        .long("duck-branch")
                        .help("Name of the branch to checkout for Duck before syncing"),
                )
                .arg(
                    Arg::new("project-repo")
                        .long("project-repo")
                        .help("HTTPS url to the single project repository"),
                )
                .arg(
                    Arg::new("project-branch")
                        .long("project-branch")
                        .help("Name of the branch to checkout for the project before syncing"),
                )
                .arg(
                    Arg::new("skip-time-filter")
                        .action(ArgAction::SetTrue)
                        .long("skip-time-filter")
                        .help("Skips commit time filter, useful to initialize a repository"),
                ),
        )
        .get_matches();

    let dry_run = matches.get_flag("dry-run");
    let verbose = matches.get_count("verbose");
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
        Some(("repo-history", subcommand_matches)) => {
            repo_history::command_repo_history(&mut io, dry_run, subcommand_matches)
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

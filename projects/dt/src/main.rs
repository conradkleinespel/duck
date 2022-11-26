use std::process::exit;

use clap::{App, AppSettings, Arg};
use log::LevelFilter;

use command::repo_history;

mod command;

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut io = rclio::RegularInputOutput::new(stdin.lock(), stdout.lock(), stderr.lock());

    let matches = App::new("dt")
        .global_setting(AppSettings::HelpExpected)
        .global_setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("Prints verbose logs")
                .global(true)
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .short('n')
                .help("Performs a trial run with no changes made")
                .global(true),
        )
        .about("Tools to manage the duck git repository")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            App::new("repo-history")
                .about("Replay history from Duck onto a single project git repository")
                .arg(
                    Arg::new("project-name-in-duck")
                        .required(true)
                        .help("The name of the project in Duck"),
                )
                .arg(
                    Arg::new("duck-repo")
                        .long("duck-repo")
                        .takes_value(true)
                        .help("HTTPS url to Duck's Git repository"),
                )
                .arg(
                    Arg::new("duck-branch")
                        .long("duck-branch")
                        .takes_value(true)
                        .help("Name of the branch to checkout for Duck before syncing"),
                )
                .arg(
                    Arg::new("project-repo")
                        .long("project-repo")
                        .takes_value(true)
                        .help("HTTPS url to the single project repository"),
                )
                .arg(
                    Arg::new("project-branch")
                        .long("project-branch")
                        .takes_value(true)
                        .help("Name of the branch to checkout for the project before syncing"),
                )
                .arg(
                    Arg::new("skip-time-filter")
                        .long("skip-time-filter")
                        .help("Skips commit time filter, useful to initialize a repository"),
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

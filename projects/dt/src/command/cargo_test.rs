use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use clap::ArgMatches;
use log::LevelFilter;
use tempfile::TempDir;

use crate::command;

const TARGET_LINUX: &'static str = "x86_64-unknown-linux-gnu";
const TARGET_WINDOWS: &'static str = "x86_64-pc-windows-gnu";

pub fn command_cargo_test(
    dry_run: bool,
    log_level: LevelFilter,
    subcommand_matches: &ArgMatches,
) -> std::result::Result<(), String> {
    let project_dir = command::arg_to_pathbuf(subcommand_matches, "project-dir").unwrap();
    let target = if subcommand_matches.is_present("windows") {
        TARGET_WINDOWS
    } else {
        TARGET_LINUX
    };

    prevent_running_dt_inside_dt(project_dir.as_path());

    let project_tmp_dir = tempfile::tempdir().unwrap();

    prepare_cross_directory(log_level, &project_dir, &project_tmp_dir, dry_run).unwrap();
    run_cross(
        project_tmp_dir.path(),
        &["test", "--target", target],
        dry_run,
    )
    .unwrap();
    cache_cross_build_objects(log_level, project_dir, project_tmp_dir, dry_run).unwrap();

    return Ok(());
}

fn prepare_cross_directory(
    log_level: LevelFilter,
    project_dir: &PathBuf,
    project_tmp_dir: &TempDir,
    dry_run: bool,
) -> io::Result<()> {
    log::info!("copying files to temporary directory for cross");
    command::rsync_files(
        project_dir.as_path(),
        project_tmp_dir.path(),
        log_level,
        dry_run,
    )
}

fn cache_cross_build_objects(
    log_level: LevelFilter,
    project_dir: PathBuf,
    project_tmp_dir: TempDir,
    dry_run: bool,
) -> io::Result<()> {
    log::info!("caching build artifacts for later test runs");
    command::rsync_files(
        project_tmp_dir
            .path()
            .to_path_buf()
            .join("target")
            .as_path(),
        project_dir.join("target").as_path(),
        log_level,
        dry_run,
    )
}

fn run_cross(current_dir: &Path, args: &[&str; 3], dry_run: bool) -> io::Result<ExitStatus> {
    log::info!("cross {}", args.join(" "));

    if dry_run {
        return Command::new("true").status();
    }

    Command::new("cross")
        .current_dir(current_dir)
        .args(args)
        .status()
}

fn prevent_running_dt_inside_dt(project: &Path) {
    log::info!("asserting that we are not running dt within dt to avoid infinite loop");

    if project.ends_with(env!("CARGO_PKG_NAME")) {
        panic!("Do not run dt from within dt");
    }
}

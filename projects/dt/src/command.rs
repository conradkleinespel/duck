use std::ffi::OsStr;
use std::fs::{create_dir, read_dir};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use clap::ArgMatches;
use log::LevelFilter;
use tempfile::TempDir;

const TARGET_LINUX: &'static str = "x86_64-unknown-linux-gnu";
const TARGET_WINDOWS: &'static str = "x86_64-pc-windows-gnu";

pub fn command_repo_funding(
    dry_run: bool,
    subcommand_matches: &ArgMatches,
) -> std::result::Result<(), String> {
    let repo_dir = arg_to_pathbuf(subcommand_matches, "repo-dir").unwrap();
    let funding_file = arg_to_pathbuf(subcommand_matches, "funding-file").unwrap();

    let github_dir = repo_dir.join(".github");
    log::info!(
        "checking github configuration directory {}",
        github_dir.display()
    );
    if github_dir.exists() {
        log::info!("github configuration directory exists");
        if !github_dir.is_dir() {
            return Err(format!(
                "unexpected file {}, expected directory",
                github_dir.display()
            ));
        }
    } else {
        log::info!("github configuration directory missing, creating");
        if !dry_run {
            create_dir(github_dir.as_path()).unwrap();
        }
    }

    log::info!(
        "copy {} to {}",
        funding_file.display(),
        github_dir.display()
    );
    if !dry_run {
        std::fs::copy(
            funding_file.as_path(),
            github_dir.join("FUNDING.yml").as_path(),
        )
        .unwrap();
    }

    return Ok(());
}

pub fn command_cargo_test(
    dry_run: bool,
    log_level: LevelFilter,
    subcommand_matches: &ArgMatches,
) -> std::result::Result<(), String> {
    let project_dir = arg_to_pathbuf(subcommand_matches, "project-dir").unwrap();
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

pub fn command_repo_rsync(
    dry_run: bool,
    log_level: LevelFilter,
    subcommand_matches: &ArgMatches,
) -> std::result::Result<(), String> {
    let project_dir = arg_to_pathbuf(subcommand_matches, "project-dir").unwrap();
    let repo_dir = arg_to_pathbuf(subcommand_matches, "repo-dir").unwrap();

    remove_dir_contents_except_git(repo_dir.as_path(), dry_run).unwrap();
    rsync_files(
        project_dir.as_path(),
        repo_dir.as_path(),
        log_level,
        dry_run,
    )
    .unwrap();

    return Ok(());
}

fn prepare_cross_directory(
    log_level: LevelFilter,
    project_dir: &PathBuf,
    project_tmp_dir: &TempDir,
    dry_run: bool,
) -> io::Result<()> {
    log::info!("copying files to temporary directory for cross");
    rsync_files(
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
    rsync_files(
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

fn rsync_files(src: &Path, dest: &Path, log_level: LevelFilter, dry_run: bool) -> io::Result<()> {
    // rsync copies directory contents only if a trailing slash is passed
    let src_str = format!("{}/", src.display().to_string());
    let dest_str = dest.display().to_string();

    log::info!("rsync {} {}", src_str, dest_str);

    let mut rsync_command = Command::new("rsync");

    if dry_run {
        rsync_command.arg("--dry-run");
    }

    if log_level >= LevelFilter::Debug {
        rsync_command.arg("--verbose").stdout(Stdio::inherit());
    } else {
        rsync_command.stdout(Stdio::null());
    }

    rsync_command
        .arg("--recursive")
        .arg("--group")
        .arg("--owner")
        .arg("--perms")
        // subcrates need to be hard-copied for cross to pickup on them
        .arg("--copy-links")
        // cargo incremental builds work based on file modification time
        .arg("--times")
        .arg(src_str)
        .arg(dest_str);

    rsync_command.status().map(|_| ())
}

fn prevent_running_dt_inside_dt(project: &Path) {
    log::info!("asserting that we are not running dt within dt to avoid infinite loop");

    if project.ends_with(env!("CARGO_PKG_NAME")) {
        panic!("Do not run dt from within dt");
    }
}

fn arg_to_pathbuf(args: &ArgMatches, arg: &str) -> io::Result<PathBuf> {
    std::fs::canonicalize(args.value_of(arg).unwrap())
}

fn remove_dir_contents_except_git(path: &Path, dry_run: bool) -> io::Result<()> {
    for entry in read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            if path.components().last().unwrap().as_os_str() == OsStr::new(".git") {
                log::info!("skipping deletion of dir {:?}", path);
            } else {
                log::info!("removing {:?}", path);
                if !dry_run {
                    std::fs::remove_dir_all(path.as_path())?;
                }
            }
        } else {
            log::info!("removing {:?}", path);
            if !dry_run {
                std::fs::remove_file(path.as_path())?;
            }
        }
    }
    Ok(())
}

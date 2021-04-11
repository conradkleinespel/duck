use clap::{App, AppSettings, Arg, ArgMatches};
use log::LevelFilter;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use tempfile::TempDir;

const TARGET_LINUX: &'static str = "x86_64-unknown-linux-gnu";
const TARGET_WINDOWS: &'static str = "x86_64-pc-windows-gnu";

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
        .about("Tools to manage the duck repository")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            App::new("cargo-test")
                .arg(
                    Arg::new("project-dir")
                        .required(true)
                        .about("Path to one of Duck's Rust projects")
                        .validator(|project_dir| read_dir(project_dir).map(|_| ())),
                )
                .arg(
                    Arg::new("windows")
                        .long("windows")
                        .short('w')
                        .about("Build for windows"),
                ),
        )
        .subcommand(
            App::new("copy-to-repo")
                .arg(
                    Arg::new("project-dir")
                        .required(true)
                        .about("Path to one of Duck's Rust projects")
                        .validator(|project_dir| read_dir(project_dir).map(|_| ())),
                )
                .arg(
                    Arg::new("repo-dir")
                        .required(true)
                        .about("Path to the repository for that Rust project")
                        .validator(|project_dir| read_dir(project_dir).map(|_| ())),
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

    match matches.subcommand() {
        Some(("cargo-test", subcommand_matches)) => {
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
        }
        Some(("copy-to-repo", subcommand_matches)) => {
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
        }
        _ => unimplemented!(),
    }
}

fn prepare_cross_directory(
    log_level: LevelFilter,
    project_dir: &PathBuf,
    project_tmp_dir: &TempDir,
    dry_run: bool,
) -> Result<()> {
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
) -> Result<()> {
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

fn run_cross(current_dir: &Path, args: &[&str; 3], dry_run: bool) -> Result<ExitStatus> {
    log::info!("cross {}", args.join(" "));

    if dry_run {
        return Command::new("true").status();
    }

    Command::new("cross")
        .current_dir(current_dir)
        .args(args)
        .status()
}

fn rsync_files(src: &Path, dest: &Path, log_level: LevelFilter, dry_run: bool) -> Result<()> {
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

fn arg_to_pathbuf(args: &ArgMatches, arg: &str) -> Result<PathBuf> {
    std::fs::canonicalize(args.value_of(arg).unwrap())
}

fn remove_dir_contents_except_git(path: &Path, dry_run: bool) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
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

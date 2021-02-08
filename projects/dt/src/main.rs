use clap::{App, AppSettings, Arg};
use log::LevelFilter;
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
                .about("Verbose logs")
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
        .get_matches();

    let verbose = matches.is_present("verbose");
    if verbose {
        env_logger::builder().filter_level(LevelFilter::Info).init();
    } else {
        env_logger::init();
    }

    log::info!("{:?}", matches);

    match matches.subcommand() {
        Some(("cargo-test", subcommand_matches)) => {
            let project_dir = subcommand_matches.value_of("project-dir").unwrap();
            let target = if subcommand_matches.is_present("windows") {
                TARGET_WINDOWS
            } else {
                TARGET_LINUX
            };

            let project_dir = std::fs::canonicalize(project_dir).unwrap();
            prevent_running_dt_inside_dt(project_dir.as_path());

            let project_tmp_dir = tempfile::tempdir().unwrap();

            prepare_cross_directory(verbose, &project_dir, &project_tmp_dir).unwrap();
            run_cross(project_tmp_dir.path(), &["test", "--target", target]).unwrap();
            cache_cross_build_objects(verbose, project_dir, project_tmp_dir).unwrap();
        }
        _ => unimplemented!(),
    }
}

fn prepare_cross_directory(
    verbose: bool,
    project_dir: &PathBuf,
    project_tmp_dir: &TempDir,
) -> Result<()> {
    log::info!("copying files to temporary directory for cross");
    rsync_files(project_dir.as_path(), project_tmp_dir.path(), verbose)
}

fn cache_cross_build_objects(
    verbose: bool,
    project_dir: PathBuf,
    project_tmp_dir: TempDir,
) -> Result<()> {
    log::info!("caching build artifacts for later test runs");
    rsync_files(
        project_tmp_dir
            .path()
            .to_path_buf()
            .join("target")
            .as_path(),
        project_dir.join("target").as_path(),
        verbose,
    )
}

fn run_cross(current_dir: &Path, args: &[&str; 3]) -> Result<ExitStatus> {
    log::info!("cross {}", args.join(" "));

    Command::new("cross")
        .current_dir(current_dir)
        .args(args)
        .status()
}

fn rsync_files(src: &Path, dest: &Path, verbose: bool) -> Result<()> {
    // rsync copies directory contents only if a trailing slash is passed
    let src_str = format!("{}/", src.display().to_string());
    let dest_str = dest.display().to_string();

    log::info!("rsync {} {}", src_str, dest_str);

    Command::new("rsync")
        .arg("--verbose")
        .arg("--recursive")
        .arg("--group")
        .arg("--owner")
        .arg("--perms")
        // subcrates need to be hard-copied for cross to pickup on them
        .arg("--copy-links")
        // cargo incremental builds work based on file modification time
        .arg("--times")
        .arg(src_str)
        .arg(dest_str)
        .stdout(if verbose {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
        .status()
        .map(|_| ())
}

fn prevent_running_dt_inside_dt(project: &Path) {
    log::info!("asserting that we are not running dt within dt to avoid infinite loop");

    if project.ends_with(env!("CARGO_PKG_NAME")) {
        panic!("Do not run dt from within dt");
    }
}

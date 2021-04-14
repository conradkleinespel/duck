use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::ArgMatches;
use log::LevelFilter;

pub mod cargo_test;
pub mod repo_history;

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
        .arg("--delete")
        .arg("--exclude=.git/")
        // subcrates need to be hard-copied for cross to pickup on them
        .arg("--copy-links")
        // cargo incremental builds work based on file modification time
        .arg("--times")
        .arg(src_str)
        .arg(dest_str);

    rsync_command.status().map(|_| ())
}

fn arg_to_pathbuf(args: &ArgMatches, arg: &str) -> io::Result<PathBuf> {
    std::fs::canonicalize(args.value_of(arg).unwrap())
}

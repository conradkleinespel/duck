use std::ffi::OsStr;
use std::fs::read_dir;
use std::io;
use std::path::Path;

use clap::ArgMatches;
use log::LevelFilter;

use crate::command;

pub fn command_repo_rsync(
    dry_run: bool,
    log_level: LevelFilter,
    subcommand_matches: &ArgMatches,
) -> std::result::Result<(), String> {
    let project_dir = command::arg_to_pathbuf(subcommand_matches, "project-dir").unwrap();
    let repo_dir = command::arg_to_pathbuf(subcommand_matches, "repo-dir").unwrap();

    remove_dir_contents_except_git(repo_dir.as_path(), dry_run).unwrap();
    command::rsync_files(
        project_dir.as_path(),
        repo_dir.as_path(),
        log_level,
        dry_run,
    )
    .unwrap();

    return Ok(());
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

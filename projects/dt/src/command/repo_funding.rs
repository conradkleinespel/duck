use std::fs::create_dir;

use clap::ArgMatches;

use crate::command;

pub fn command_repo_funding(
    dry_run: bool,
    subcommand_matches: &ArgMatches,
) -> std::result::Result<(), String> {
    let repo_dir = command::arg_to_pathbuf(subcommand_matches, "repo-dir").unwrap();
    let funding_file = command::arg_to_pathbuf(subcommand_matches, "funding-file").unwrap();

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

use crate::command::rsync_files;
use chrono::NaiveDateTime;
use clap::ArgMatches;
use git2::build::CheckoutBuilder;
use git2::{
    Cred, Error, FetchOptions, IndexAddOption, PushOptions, RemoteCallbacks, Repository, Sort,
};
use log::LevelFilter;
use rclio::{CliInputOutput, RegularInputOutput};
use std::borrow::Borrow;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn command_repo_history(
    io: &mut RegularInputOutput,
    dry_run: bool,
    log_level: LevelFilter,
    subcommand_matches: &ArgMatches,
) -> std::result::Result<(), String> {
    let duck_repo_url = subcommand_matches
        .get_one::<String>("duck-repo")
        .map(|s| s.to_string())
        .unwrap_or("https://github.com/conradkleinespel/duck.git".to_string());
    let duck_branch = subcommand_matches
        .get_one::<String>("duck-branch")
        .map(|s| s.to_string())
        .unwrap_or("master".to_string());
    let project_name_in_duck = subcommand_matches
        .get_one::<String>("project-name-in-duck")
        .unwrap();
    let default_project_repo_url = format!(
        "https://github.com/conradkleinespel/{}.git",
        project_name_in_duck
    );
    let project_repo_url = subcommand_matches
        .get_one::<String>("project-repo")
        .map(|s| s.to_string())
        .unwrap_or(default_project_repo_url.as_str().to_string());
    let project_branch = subcommand_matches
        .get_one::<String>("project-branch")
        .map(|s| s.to_string())
        .unwrap_or("master".to_string());
    let skip_time_filter = subcommand_matches.get_flag("skip-time-filter");

    let git_tmp_dir = tempfile::tempdir().unwrap();
    let git_tmp_dir_path = git_tmp_dir.path().to_path_buf();

    log::info!("creating tmp directory {}", git_tmp_dir_path.display());

    let duck_path = git_tmp_dir_path.join("duck");
    let project_path = git_tmp_dir_path.join("project");

    let (git_username, git_password) = get_username_and_password(io).unwrap();

    log::info!("cloning {}", duck_repo_url);
    let mut duck_repo =
        git2::Repository::clone(duck_repo_url.as_str(), duck_path.as_path()).unwrap();
    checkout_branch(
        &mut duck_repo,
        duck_branch.as_str(),
        git_username.as_str(),
        git_password.as_str(),
    )
    .unwrap();
    log::info!("cloning {}", project_repo_url);
    let mut project_repo =
        git2::Repository::clone(project_repo_url.as_str(), project_path.as_path()).unwrap();
    checkout_branch(
        &mut project_repo,
        project_branch.as_str(),
        git_username.as_str(),
        git_password.as_str(),
    )
    .unwrap();

    match replay_all_commits(
        log_level,
        project_name_in_duck,
        &mut duck_repo,
        duck_path.as_path(),
        &mut project_repo,
        project_branch.as_str(),
        project_path.as_path(),
        skip_time_filter,
    ) {
        Err(err) => Result::Err(err).unwrap(),
        Ok(num_commits_replayed) => {
            if num_commits_replayed == 0 {
                log::info!("no commits replayed, skipping git-push");
                return Ok(());
            }
        }
    }

    let branch_name = format!(
        "duck-sync-{}",
        project_repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string()
    );
    let push_refspec = format!("refs/heads/master:refs/heads/{}", branch_name);
    log::info!("pusing refspec {}", push_refspec);
    let mut remote_callbacks = RemoteCallbacks::new();
    remote_callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        log::info!("authenticating before git-push");
        Cred::userpass_plaintext(git_username.as_str(), git_password.as_str())
    });

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(remote_callbacks);

    log::info!(
        "pushing to remote remote {:?} {:?}",
        project_repo.find_remote("origin").unwrap().name(),
        project_repo.find_remote("origin").unwrap().url()
    );

    if !dry_run {
        project_repo
            .find_remote("origin")
            .unwrap()
            .push(&[push_refspec.as_str()], Some(&mut push_options))
            .unwrap();
    }

    log::info!("check state of branch {}", branch_name);

    Ok(())
}

fn replay_all_commits(
    log_level: LevelFilter,
    project_name_in_duck: &str,
    duck_repo: &mut Repository,
    duck_path: &Path,
    project_repo: &mut Repository,
    project_branch: &str,
    project_path: &Path,
    skip_time_filter: bool,
) -> Result<u64, Error> {
    let mut num_commits_replayed = 0;

    let mut revwalk = duck_repo.revwalk().unwrap();
    revwalk.set_sorting(Sort::TIME | Sort::REVERSE).unwrap();
    revwalk.push_head().unwrap();

    let remote_project_branch_refspec = format!("refs/remotes/origin/{}", project_branch);

    // We want the "Author Date" and not the "Commit Date" because the "Author Date" is earliest,
    // this guarantees we don't miss any commits, looking at already synced commits is OK because
    // commits with empty changelist are skipped
    let project_repo_last_commit_time = project_repo
        .revparse_single(remote_project_branch_refspec.as_str())
        .unwrap()
        .as_commit()
        .unwrap()
        .author()
        .when()
        .seconds();

    log::info!("project branch {}", remote_project_branch_refspec);
    log::info!(
        "last commit time {:?} {:?}",
        project_repo
            .revparse_single(remote_project_branch_refspec.as_str())
            .unwrap()
            .as_commit(),
        NaiveDateTime::from_timestamp_opt(project_repo_last_commit_time, 0)
    );

    let commits = revwalk.filter_map(|oid| {
        let commit = duck_repo.find_commit(oid.unwrap()).unwrap();

        if commit.parent_count() > 1 {
            log::info!(
                "skipping merge commit {} {:?}",
                commit.id(),
                commit.message()
            );
            return None;
        }

        if commit
            .tree()
            .unwrap()
            .get_path(
                PathBuf::new()
                    .join("projects")
                    .join(project_name_in_duck)
                    .as_path(),
            )
            .is_err()
        {
            log::info!(
                "skipping commit not containing {} project {} {:?}",
                project_name_in_duck,
                commit.id(),
                commit.message()
            );
            return None;
        }

        if !skip_time_filter && commit.time().seconds() < project_repo_last_commit_time {
            log::info!(
                "skipping commit earlier than HEAD of {} project {} {:?} {:?}",
                project_name_in_duck,
                commit.id(),
                commit.message(),
                NaiveDateTime::from_timestamp_opt(commit.time().seconds(), 0)
            );
            return None;
        }

        return Some(commit);
    });

    for commit in commits {
        let last_commit = project_repo.head().unwrap().peel_to_commit().unwrap();

        log::info!("checking out commit {} {:?}", commit.id(), commit.message());
        duck_repo
            .checkout_tree(commit.as_object(), Some(CheckoutBuilder::new().force()))
            .unwrap();

        // Wait for checkout to complete on local filesystem (no idea why I need this, but it doesn't work otherwise)
        std::thread::sleep(Duration::from_millis(1000));

        log::info!("syncing files from duck to {}", project_name_in_duck);
        rsync_files(
            duck_path
                .to_path_buf()
                .join("projects")
                .join(project_name_in_duck)
                .as_path(),
            project_path,
            log_level,
            false,
        )
        .unwrap();

        log::info!("adding files to index");
        let mut project_index = project_repo.index().unwrap();
        project_index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        project_index.write().unwrap();

        let tree = project_repo
            .find_tree(project_index.write_tree().unwrap())
            .unwrap();
        if project_repo
            .diff_tree_to_tree(
                Some(last_commit.tree().unwrap().borrow()),
                Some(&tree),
                None,
            )
            .unwrap()
            .deltas()
            .len()
            == 0
        {
            log::info!("skipping empty commit");
            continue;
        }

        log::info!(
            "apply commit {:?} with time {:?}",
            commit,
            NaiveDateTime::from_timestamp_opt(commit.time().seconds(), 0)
        );

        project_repo
            .commit(
                Some("HEAD"),
                commit.author().borrow(),
                commit.committer().borrow(),
                String::from_utf8_lossy(commit.message_bytes()).as_ref(),
                tree.borrow(),
                &[last_commit.borrow()],
            )
            .map(|_| ())?;

        num_commits_replayed += 1;
    }

    Ok(num_commits_replayed)
}

fn checkout_branch(
    repo: &mut Repository,
    branch: &str,
    git_username: &str,
    git_password: &str,
) -> Result<(), Error> {
    let mut remote_callbacks = RemoteCallbacks::new();
    remote_callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        log::info!("authenticating before git-checkout");
        Cred::userpass_plaintext(git_username, git_password)
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(remote_callbacks);

    let remote_branch_refspec = format!("refs/remotes/origin/{}", branch);
    repo.find_remote("origin")
        .unwrap()
        .download(&[remote_branch_refspec.as_str()], Some(&mut fetch_options))
        .unwrap();

    let commit_obj = repo
        .revparse_single(remote_branch_refspec.as_str())
        .unwrap();

    repo.checkout_tree(&commit_obj, Some(CheckoutBuilder::new().force()))
        .unwrap();

    return Ok(());
}

fn get_username_and_password(io: &mut RegularInputOutput) -> Result<(String, String), Error> {
    let git_username =
        std::env::var("DUCK_USERNAME").unwrap_or_else(|_| io.prompt_line("Username: ").unwrap());
    let git_password = std::env::var("DUCK_PASSWORD").unwrap_or_else(|_| {
        io.prompt_password("Access token: ")
            .map(|s| s.into_inner())
            .unwrap()
    });

    return Ok((git_username, git_password));
}

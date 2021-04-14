use crate::command::rsync_files;
use crate::rclio::{CliInputOutput, RegularInputOutput};
use clap::ArgMatches;
use git2::{
    Cred, Error, IndexAddOption, PushOptions, RemoteCallbacks, Repository, ResetType, Sort,
};
use log::LevelFilter;
use std::borrow::Borrow;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn command_repo_history(
    io: &mut RegularInputOutput,
    dry_run: bool,
    log_level: LevelFilter,
    subcommand_matches: &ArgMatches,
) -> std::result::Result<(), String> {
    let duck_repo_url = subcommand_matches.value_of("duck-repo").unwrap();
    let project_name_in_duck = subcommand_matches.value_of("project-name-in-duck").unwrap();
    let project_repo_url = subcommand_matches.value_of("project-repo").unwrap();

    let git_tmp_dir = tempfile::tempdir().unwrap();
    let git_tmp_dir_path = git_tmp_dir.path().to_path_buf();

    log::info!("creating tmp directory {}", git_tmp_dir_path.display());

    let duck_path = git_tmp_dir_path.join("duck");
    let project_path = git_tmp_dir_path.join("project");

    log::info!("cloning {}", duck_repo_url);
    let mut duck_repo = git2::Repository::clone(duck_repo_url, duck_path.as_path()).unwrap();
    log::info!("cloning {}", project_repo_url);
    let mut project_repo =
        git2::Repository::clone(project_repo_url, project_path.as_path()).unwrap();

    replay_all_commits(
        log_level,
        project_name_in_duck,
        &mut duck_repo,
        duck_path.as_path(),
        &mut project_repo,
        project_path.as_path(),
    )
    .unwrap();

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
    let push_refspec = format!("refs/heads/master:refs/heads/origin/{}", branch_name);
    log::info!("pusing refspec {}", push_refspec);
    let mut remote_callbacks = RemoteCallbacks::new();
    remote_callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        log::info!("authenticating before git-push");

        let git_username = io.prompt_line("Username: ").unwrap();
        let git_password = io.prompt_password("Access token: ").unwrap();

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
    project_path: &Path,
) -> Result<(), Error> {
    let mut revwalk = duck_repo.revwalk().unwrap();
    revwalk.set_sorting(Sort::TIME | Sort::REVERSE).unwrap();
    revwalk.push_head().unwrap();

    let project_repo_last_commit_time = project_repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .time()
        .seconds();

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

        if commit.time().seconds() < project_repo_last_commit_time {
            log::info!(
                "skipping commit earlier than HEAD of {} project {} {:?}",
                project_name_in_duck,
                commit.id(),
                commit.message()
            );
            return None;
        }

        return Some(commit);
    });

    for commit in commits {
        // TODO: test what happens if project_repo is empty, error BareRepo?
        let last_commit = project_repo.head().unwrap().peel_to_commit().unwrap();

        log::info!("checking out commit {} {:?}", commit.id(), commit.message());
        duck_repo
            .reset(commit.as_object(), ResetType::Hard, None)
            .unwrap();

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
    }

    Ok(())
}

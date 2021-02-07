mod rclio;
mod rutil;

use crate::rclio::{CliInputOutput, RegularInputOutput};
use clap::{App, AppSettings, Arg};
use git2::{Cred, RemoteCallbacks, Repository, StatusOptions};
use tempfile::tempdir;

fn main() {
    let matches = App::new("duck-tools")
        .global_setting(AppSettings::HelpRequired)
        .global_setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("duck-tools helps manage the duck monorepo")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            App::new("push")
                .about("Push changes from a project to it's repository")
                .arg(
                    Arg::new("src-project")
                        .about("The name of the project")
                        .required(true),
                )
                .arg(
                    Arg::new("dst-repo")
                        .about("The URL of the repository to push to")
                        .required(true),
                ),
        )
        .get_matches();

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut io = RegularInputOutput::new(stdin.lock(), stdout.lock(), stderr.lock());

    match matches.subcommand() {
        Some(("push", submatches)) => {
            println!("{}", submatches.value_of("src-project").unwrap());
            println!("{}", submatches.value_of("dst-repo").unwrap());

            let dst_repo_temp_dir = tempdir().unwrap();
            let dst_repo_url = submatches.value_of("dst-repo").unwrap();

            let ssh_passphrase = io.prompt_password("SSH passphrase: ").unwrap();

            let mut callbacks = RemoteCallbacks::new();
            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                Cred::ssh_key(
                    username_from_url.unwrap(),
                    None,
                    std::path::Path::new(&format!(
                        "{}/.ssh/id_rsa",
                        std::env::var("HOME").unwrap()
                    )),
                    Some(ssh_passphrase.as_str()),
                )
            });

            let mut fo = git2::FetchOptions::new();
            fo.remote_callbacks(callbacks);

            let repo = git2::build::RepoBuilder::new()
                .fetch_options(fo)
                .clone(dst_repo_url, dst_repo_temp_dir.path())
                .unwrap();

            // let mut index = repo.index().unwrap();
            // index.add_path(dst_repo_temp_dir.path()).unwrap();

            for status_entry in repo
                .statuses(Some(StatusOptions::new().include_unmodified(true)))
                .unwrap()
                .iter()
            {
                println!("{:?}", status_entry.path());
            }
        }
        _ => unimplemented!(),
    }
}

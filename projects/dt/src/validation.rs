use std::ffi::OsStr;
use std::fs::{read_dir, read_to_string};
use std::io;

pub fn validate_dir(path: &str) -> io::Result<()> {
    read_dir(path).map(|_| ())
}

pub fn validate_file(path: &str) -> io::Result<()> {
    read_to_string(path).map(|_| ())
}

pub fn validate_git_repo(path: &str) -> std::result::Result<(), String> {
    for entry in read_dir(path).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            if path.components().last().unwrap().as_os_str() == OsStr::new(".git") {
                return Ok(());
            }
        }
    }

    return Err(format!("repo-dir must contain a .git directory"));
}

extern crate libc;
extern crate rooster;

use rooster::io::{RegularInput, RegularOutput};
use std::env::VarError;
use std::path::PathBuf;

const ROOSTER_FILE_ENV_VAR: &'static str = "ROOSTER_FILE";
const ROOSTER_FILE_DEFAULT: &'static str = ".passwords.rooster";

fn get_password_file_path() -> Result<PathBuf, i32> {
    // First, look for the ROOSTER_FILE environment variable.
    match std::env::var(ROOSTER_FILE_ENV_VAR) {
        Ok(filename) => Ok(PathBuf::from(filename)),
        Err(VarError::NotPresent) => {
            // If the environment variable is not there, we'll look in the default location:
            // ~/.passwords.rooster
            let mut file_default = PathBuf::from(
                dirs::home_dir()
                    .ok_or(1)?
                    .as_os_str()
                    .to_os_string()
                    .into_string()
                    .map_err(|_| 1)?,
            );
            file_default.push(ROOSTER_FILE_DEFAULT);
            Ok(file_default)
        }
        Err(VarError::NotUnicode(_)) => Err(1),
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let args_refs = args.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

    let rooster_file_path = get_password_file_path().unwrap_or_else(|err| std::process::exit(err));

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let tty = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/tty")
        .unwrap();

    std::process::exit(rooster::main_with_args(
        args_refs.as_slice(),
        &mut RegularInput {
            stdin_lock: stdin.lock(),
        },
        &mut RegularOutput {
            stdout_lock: stdout.lock(),
            stderr_lock: stderr.lock(),
            tty,
        },
        &rooster_file_path,
    ));
}

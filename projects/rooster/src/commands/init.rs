// Copyright 2014-2017 The Rooster Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use io::{CliReader, CliWriter};
use io::{OutputType, Style};
use std::path::PathBuf;

pub fn callback_exec(
    matches: &clap::ArgMatches,
    reader: &mut impl CliReader,
    writer: &mut impl CliWriter,
    rooster_file_path: &PathBuf,
) -> Result<(), i32> {
    let filename_as_string = rooster_file_path.to_string_lossy().into_owned();
    if rooster_file_path.exists() && !matches.is_present("force-for-tests") {
        writer.writeln(
            Style::error("Woops, there is already a Rooster file located at:"),
            OutputType::Error,
        );
        writer.writeln(
            Style::error(format!("    {}", filename_as_string)),
            OutputType::Error,
        );
        writer.nl(OutputType::Error);
        writer.writeln(
            Style::error("Type `rooster --help` to see what Rooster can do for you."),
            OutputType::Error,
        );
        return Err(1);
    }

    writer.writeln(Style::title("Welcome to Rooster"), OutputType::Standard);
    writer.nl(OutputType::Standard);
    writer.writeln(Style::info("Rooster is a simple password manager for geeks. Let's get started! Type ENTER to continue."), OutputType::Standard);

    if let Err(err) = reader.read_line() {
        writer.writeln(
            Style::error(format!(
                "Woops, I didn't see the ENTER key (reason: {:?}).",
                err
            )),
            OutputType::Error,
        );
        return Err(1);
    }

    writer.writeln(Style::title("The master password"), OutputType::Standard);
    writer.nl(OutputType::Standard);
    writer.writeln(Style::info(
        "With Rooster, you only need to remember one password: \
    the master password. It keeps all of you other passwords safe. The stronger it is, the better your passwords are \
                      protected."
    ), OutputType::Standard);
    writer.nl(OutputType::Standard);

    writer.write("Choose your master password: ", OutputType::Standard);
    let master_password = reader.read_password().map_err(|err| {
        writer.writeln(
            Style::error(format!(
                "Woops, I couldn't read the master passwords ({:?}).",
                err
            )),
            OutputType::Error,
        );
        1
    })?;

    if master_password.len() == 0 {
        writer.writeln(
            Style::error("Your master password cannot be empty."),
            OutputType::Error,
        );
        return Err(1);
    }

    let store = match ::password::v2::PasswordStore::new(master_password) {
        Ok(store) => store,
        Err(err) => {
            writer.writeln(
                Style::error(format!(
                    "Woops, I couldn't use the random number generator on your machine \
                     (reason: {:?}). Without it, I can't create a secure password file.",
                    err
                )),
                OutputType::Error,
            );
            return Err(1);
        }
    };

    let mut file = match ::create_password_file(filename_as_string.as_str()).map_err(|_| 1) {
        Ok(file) => file,
        Err(err) => {
            writer.writeln(
                Style::error(format!(
                    "Woops, I couldn't create a new password file (reason: {:?})",
                    err
                )),
                OutputType::Error,
            );
            return Err(1);
        }
    };

    if let Err(err) = store.sync(&mut file) {
        if let Err(err) = ::std::fs::remove_file(rooster_file_path) {
            writer.writeln(
                Style::error(format!(
                    "Woops, I was able to create a new password file but couldn't save \
                     it (reason: {:?}). You may want to remove this dangling file:",
                    err
                )),
                OutputType::Error,
            );
            writer.writeln(
                Style::error(format!("    {}", filename_as_string)),
                OutputType::Error,
            );
            return Err(1);
        }
        writer.writeln(
            Style::error(format!(
                "Woops, I couldn't create a new password file (reason: {:?}).",
                err
            )),
            OutputType::Error,
        );
        return Err(1);
    }

    writer.nl(OutputType::Standard);
    writer.writeln(
        Style::title("All done and ready to rock"),
        OutputType::Standard,
    );
    writer.nl(OutputType::Standard);
    writer.writeln(
        Style::success("You passwords will be saved in:"),
        OutputType::Standard,
    );
    writer.writeln(
        Style::success(format!("    {}", filename_as_string)),
        OutputType::Standard,
    );
    writer.nl(OutputType::Standard);
    writer.writeln(
        Style::info(
            "If you wish to change the location of your password file, you can set it in the \
        ROOSTER_FILE environment variable. For instance:",
        ),
        OutputType::Standard,
    );
    writer.writeln(
        Style::info("    export ROOSTER_FILE=path/to/passwords.rooster"),
        OutputType::Standard,
    );
    writer.nl(OutputType::Standard);
    writer.writeln(
        Style::info("Type `rooster --help` to see what Rooster can do for you."),
        OutputType::Standard,
    );

    Ok(())
}

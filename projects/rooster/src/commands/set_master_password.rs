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
use password;
use std::ops::Deref;

pub fn callback_exec(
    _matches: &clap::ArgMatches,
    store: &mut password::v2::PasswordStore,
    reader: &mut impl CliReader,
    writer: &mut impl CliWriter,
) -> Result<(), i32> {
    writer.write("Type your new master password: ", OutputType::Standard);
    match reader.read_password() {
        Ok(master_password) => {
            writer.write(
                "Type your new master password once more: ",
                OutputType::Standard,
            );
            let master_password_confirmation = match reader.read_password() {
                Ok(master_password_confirmation) => master_password_confirmation,
                Err(err) => {
                    writer.writeln(
                        Style::error(format!(
                            "I could not read your new master password (reason: {:?}).",
                            err
                        )),
                        OutputType::Error,
                    );
                    return Err(1);
                }
            };

            if master_password != master_password_confirmation {
                writer.writeln(
                    Style::error("The master password confirmation did not match. Aborting."),
                    OutputType::Error,
                );
                return Err(1);
            }

            store.change_master_password(master_password.deref());
        }
        Err(err) => {
            writer.writeln(
                Style::error(format!(
                    "I could not read your new master password (reason: {:?}).",
                    err
                )),
                OutputType::Error,
            );
            return Err(1);
        }
    }
    writer.writeln(
        Style::success("Your master password has been changed."),
        OutputType::Standard,
    );
    Ok(())
}

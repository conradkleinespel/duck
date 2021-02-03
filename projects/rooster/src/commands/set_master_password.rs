use crate::io::{CliReader, CliWriter};
use crate::io::{OutputType, Style};
use crate::password;
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

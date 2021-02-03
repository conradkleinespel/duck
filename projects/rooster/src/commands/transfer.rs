use ffi;
use io::{CliReader, CliWriter};
use io::{OutputType, Style};
use list;
use password;

pub fn callback_exec(
    matches: &clap::ArgMatches,
    store: &mut password::v2::PasswordStore,
    reader: &mut impl CliReader,
    writer: &mut impl CliWriter,
) -> Result<(), i32> {
    let query = matches.value_of("app").unwrap();
    let new_username = matches.value_of("new_username").unwrap().to_owned();

    let password = list::search_and_choose_password(
        store,
        query,
        list::WITH_NUMBERS,
        "Which password would you like to transfer?",
        reader,
        writer,
    )
    .ok_or(1)?
    .clone();

    let old_username = password.username;

    let change_result =
        store.change_password(&password.name, &|old_password: password::v2::Password| {
            password::v2::Password {
                name: old_password.name.clone(),
                username: new_username.clone(),
                password: old_password.password.clone(),
                created_at: old_password.created_at,
                updated_at: ffi::time(),
            }
        });

    match change_result {
        Ok(_) => {
            writer.writeln(
                Style::success(format!(
                    "Done! I've transfered {} to {}",
                    old_username, new_username
                )),
                OutputType::Standard,
            );
            Ok(())
        }
        Err(err) => {
            writer.writeln(
                Style::error(format!(
                    "Woops, I couldn't save the new app name (reason: {:?}).",
                    err
                )),
                OutputType::Error,
            );
            Err(1)
        }
    }
}

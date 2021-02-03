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
    let new_name = matches.value_of("new_name").unwrap().to_owned();

    let password = list::search_and_choose_password(
        store,
        query,
        list::WITH_NUMBERS,
        "Which password would you like to rename?",
        reader,
        writer,
    )
    .ok_or(1)?
    .clone();

    let change_result =
        store.change_password(&password.name, &|old_password: password::v2::Password| {
            password::v2::Password {
                name: new_name.clone(),
                username: old_password.username.clone(),
                password: old_password.password.clone(),
                created_at: old_password.created_at,
                updated_at: ffi::time(),
            }
        });

    match change_result {
        Ok(_) => {
            writer.writeln(
                Style::success(format!(
                    "Done! I've renamed {} to {}",
                    password.name, new_name
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

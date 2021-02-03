use clip;
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

    let password = list::search_and_choose_password(
        store,
        query,
        list::WITH_NUMBERS,
        "Which password would like to update?",
        reader,
        writer,
    )
    .ok_or(1)?
    .clone();

    writer.write(
        format!("What password do you want for \"{}\"? ", password.name),
        OutputType::Standard,
    );
    let password_as_string = reader.read_password().map_err(|err| {
        writer.writeln(
            Style::error(format!(
                "\nI couldn't read the app's password (reason: {:?}).",
                err
            )),
            OutputType::Error,
        );
        1
    })?;

    let password = store
        .change_password(&password.name, &|old_password: password::v2::Password| {
            password::v2::Password {
                name: old_password.name,
                username: old_password.username,
                password: password_as_string.clone(),
                created_at: old_password.created_at,
                updated_at: ffi::time(),
            }
        })
        .map_err(|err| {
            writer.writeln(
                Style::error(format!(
                    "Woops, I couldn't save the new password (reason: {:?}).",
                    err
                )),
                OutputType::Error,
            );
            1
        })?;

    let show = matches.is_present("show");
    clip::confirm_password_retrieved(show, &password, writer);
    Ok(())
}

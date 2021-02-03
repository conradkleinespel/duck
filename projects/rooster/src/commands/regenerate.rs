use clip;
use ffi;
use generate::{check_password_len, PasswordSpec};
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
        "Which password would you like to regenerate?",
        reader,
        writer,
    )
    .ok_or(1)?
    .clone();

    let pwspec = PasswordSpec::new(
        matches.is_present("alnum"),
        matches
            .value_of("length")
            .and_then(|len| check_password_len(len.parse::<usize>().ok(), writer)),
    );

    let password_as_string = match pwspec.generate_hard_password() {
        Ok(password_as_string) => password_as_string,
        Err(io_err) => {
            writer.writeln(
                Style::error(format!(
                    "Woops, I could not generate the password (reason: {:?}).",
                    io_err
                )),
                OutputType::Error,
            );
            return Err(1);
        }
    };

    let change_result =
        store.change_password(&password.name, &|old_password: password::v2::Password| {
            password::v2::Password {
                name: old_password.name.clone(),
                username: old_password.username.clone(),
                password: password_as_string.clone(),
                created_at: old_password.created_at,
                updated_at: ffi::time(),
            }
        });

    match change_result {
        Ok(password) => {
            let show = matches.is_present("show");
            clip::confirm_password_retrieved(show, &password, writer);
            Ok(())
        }
        Err(err) => {
            writer.writeln(
                Style::error(format!(
                    "Woops, I couldn't save the new password (reason: {:?}).",
                    err
                )),
                OutputType::Error,
            );
            Err(1)
        }
    }
}

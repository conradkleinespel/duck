use crate::io::{CliReader, CliWriter};
use crate::io::{OutputType, Style};
use crate::list;
use crate::password;

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
        "Which password would you like me to delete?",
        reader,
        writer,
    )
    .ok_or(1)?
    .clone();

    if let Err(err) = store.delete_password(&password.name) {
        writer.writeln(
            Style::error(format!(
                "Woops, I couldn't delete this password (reason: {:?}).",
                err
            )),
            OutputType::Error,
        );
        return Err(1);
    }

    writer.writeln(
        Style::success(format!(
            "Done! I've deleted the password for \"{}\".",
            password.name
        )),
        OutputType::Standard,
    );

    Ok(())
}

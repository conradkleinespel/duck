use clip;

use io::{CliReader, CliWriter};
use list;
use password;

pub fn callback_exec(
    matches: &clap::ArgMatches,
    store: &mut password::v2::PasswordStore,
    reader: &mut impl CliReader,
    writer: &mut impl CliWriter,
) -> Result<(), i32> {
    let show = matches.is_present("show");
    let query = matches.value_of("app").unwrap();

    let prompt = format!(
        "Which password would you like {}? ",
        if show {
            "to see"
        } else {
            "to copy to your clipboard"
        },
    );
    let password =
        list::search_and_choose_password(store, query, list::WITH_NUMBERS, &prompt, reader, writer)
            .ok_or(1)?;

    clip::confirm_password_retrieved(show, &password, writer);

    Ok(())
}

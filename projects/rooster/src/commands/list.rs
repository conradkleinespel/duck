use io::{CliReader, CliWriter};
use io::{OutputType, Style};
use list;
use password;

pub fn callback_exec(
    _matches: &clap::ArgMatches,
    store: &mut password::v2::PasswordStore,
    _reader: &mut impl CliReader,
    writer: &mut impl CliWriter,
) -> Result<(), i32> {
    let passwords = store.get_all_passwords();

    if passwords.len() == 0 {
        writer.writeln(
            Style::info("No passwords on record yet. Add one with `rooster add <app> <username>`."),
            OutputType::Standard,
        );
    } else {
        list::print_list_of_passwords(&passwords, list::WITHOUT_NUMBERS, writer);
    }

    Ok(())
}

use io::{CliReader, CliWriter, OutputType, Style};
use password;

pub fn callback_exec(
    matches: &clap::ArgMatches,
    store: &mut password::v2::PasswordStore,
    _reader: &mut impl CliReader,
    writer: &mut impl CliWriter,
) -> Result<(), i32> {
    let log2_n = matches
        .value_of("log2n")
        .unwrap()
        .trim()
        .parse::<u8>()
        .unwrap();
    let r = matches
        .value_of("r")
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();
    let p = matches
        .value_of("p")
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();

    if log2_n <= 0 || r <= 0 || p <= 0 {
        writer.writeln(
            Style::error(format!(
                "The parameters must be > 0 ({}, {}, {})",
                log2_n, r, p
            )),
            OutputType::Error,
        );
        return Err(1);
    }

    if !matches.is_present("force") && (log2_n > 20 || r > 8 || p > 1) {
        writer.writeln(
            Style::error("These parameters seem very high. You might be unable to open your password file ever again. Aborting."),
            OutputType::Error);
        writer.writeln(
            Style::error(
                "Run with --force to force, but make a backup of your password file first.",
            ),
            OutputType::Error,
        );
        return Err(1);
    }

    store.change_scrypt_params(log2_n, r, p);

    Ok(())
}

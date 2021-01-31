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

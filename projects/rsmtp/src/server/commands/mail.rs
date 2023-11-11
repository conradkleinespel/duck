// Copyright 2014 The Rustastic SMTP Developers
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

use super::super::super::common::mailbox::Mailbox;
use super::super::super::common::stream::InputStream;
use super::super::super::common::stream::OutputStream;
use super::super::Command;
use super::super::NextMiddleware;
use super::super::ServerConfig;
use super::HeloSeen;
use super::MailHandler;
use std::net::TcpStream;

type Next<CT> = Option<NextMiddleware<CT, TcpStream>>;
type Input = InputStream<TcpStream>;
type Output = OutputStream<TcpStream>;

fn check_state<CT: HeloSeen>(
    config: &ServerConfig<CT>,
    container: &mut CT,
    input: &mut Input,
    output: &mut Output,
    line: &str,
    next: Next<CT>,
) {
    match container.helo_seen() {
        false => {
            output
                .write_line("503 Bad sequence of commands, HELO/EHLO first")
                .unwrap();
        }
        true => {
            next.unwrap().call(config, container, input, output, line);
        }
    }
}

fn check_mailbox_format<CT>(
    config: &ServerConfig<CT>,
    container: &mut CT,
    input: &mut Input,
    output: &mut Output,
    line: &str,
    next: Next<CT>,
) {
    match line.len() < 2 || line.starts_with("<") || line.ends_with(">") {
        false => {
            output
                .write_line("501 Invalid argument, format: '<email@example.com>'")
                .unwrap();
        }
        true => {
            next.unwrap().call(config, container, input, output, line);
        }
    }
}

fn handle_no_sender<CT: MailHandler>(
    config: &ServerConfig<CT>,
    container: &mut CT,
    input: &mut Input,
    output: &mut Output,
    line: &str,
    next: Next<CT>,
) {
    match line == "<>" {
        true => match container.handle_sender_address(None) {
            Ok(_) => {
                output.write_line("250 OK").unwrap();
            }
            Err(_) => {
                output.write_line("550 Mailbox not taken").unwrap();
            }
        },
        false => {
            next.unwrap().call(config, container, input, output, line);
        }
    }
}

fn handle_sender<CT: MailHandler>(
    _: &ServerConfig<CT>,
    container: &mut CT,
    _: &mut Input,
    output: &mut Output,
    line: &str,
    _: Next<CT>,
) {
    match Mailbox::parse(&line[1..line.len() - 1]) {
        Err(err) => {
            output
                .write_line(format!("553 Email address invalid: {:?}", err).as_ref())
                .unwrap();
        }
        Ok(mailbox) => match container.handle_sender_address(Some(mailbox)) {
            Ok(_) => {
                output.write_line("250 OK").unwrap();
            }
            Err(_) => {
                output.write_line("550 Mailbox not taken").unwrap();
            }
        },
    }
}

/// Returns the MAIL command
pub fn get<CT: HeloSeen + MailHandler + Clone + Send>() -> Command<CT, TcpStream> {
    let mut command = Command::new();
    command.starts_with("MAIL FROM:");
    command.middleware(check_state);
    command.middleware(check_mailbox_format);
    command.middleware(handle_no_sender);
    command.middleware(handle_sender);
    command
}

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

use super::super::super::common::stream::InputStream;
use super::super::super::common::stream::OutputStream;
use super::super::super::common::utils;
use super::super::Command;
use super::super::NextMiddleware;
use super::super::ServerConfig;
use super::HeloHandler;
use super::HeloSeen;
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
        true => {
            output
                .write_line("503 Bad sequence of commands, HELO/EHLO already seen")
                .unwrap();
        }
        false => {
            next.unwrap().call(config, container, input, output, line);
        }
    }
}

fn check_domain<CT>(
    config: &ServerConfig<CT>,
    container: &mut CT,
    input: &mut Input,
    output: &mut Output,
    line: &str,
    next: Next<CT>,
) {
    match utils::get_domain(line) {
        None => {
            output.write_line("501 Domain name is invalid").unwrap();
        }
        Some(domain) => match domain.len() == line.len() {
            false => {
                output.write_line("501 Domain name is invalid").unwrap();
            }
            true => {
                next.unwrap().call(config, container, input, output, line);
            }
        },
    }
}

fn handle_domain<CT: HeloSeen + HeloHandler>(
    config: &ServerConfig<CT>,
    container: &mut CT,
    _: &mut Input,
    output: &mut Output,
    line: &str,
    _: Next<CT>,
) {
    match container.handle_domain(line) {
        Ok(_) => {
            container.set_helo_seen(true);
            let mut i = config.extensions.len();
            let host = if i > 0 {
                format!("250-{}", config.hostname)
            } else {
                format!("250 {}", config.hostname)
            };
            output.write_line(host.as_ref()).unwrap();
            while i != 1 {
                output
                    .write_line(format!("250-{}", config.extensions[i - 1]).as_ref())
                    .unwrap();
                i -= 1;
            }
            if i == 1 {
                output
                    .write_line(format!("250 {}", config.extensions[i - 1]).as_ref())
                    .unwrap();
            }
        }
        Err(_) => {
            output.write_line("550 Domain not taken").unwrap();
        }
    }
}

/// Returns the MAIL command
pub fn get<CT: HeloSeen + HeloHandler + Clone + Send>() -> Command<CT, TcpStream> {
    let mut command = Command::new();
    command.starts_with("EHLO ");
    command.middleware(check_state);
    command.middleware(check_domain);
    command.middleware(handle_domain);
    command
}

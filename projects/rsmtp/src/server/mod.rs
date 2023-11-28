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

//! The `server` module contains things needed to build an SMTP server,
//! but useless for an SMTP client.

extern crate libc;

use super::common::stream::{InputStream, OutputStream};
use std::borrow::ToOwned;
use std::clone::Clone;
use std::io::Result as IoResult;
use std::net::IpAddr;
use std::net::{TcpListener, TcpStream};
use std::ops::Deref;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::sync::Arc;
use std::thread;

/// Core SMTP commands
pub mod commands;

extern "C" {
    fn gethostname(name: *mut libc::c_char, size: libc::size_t) -> libc::c_int;
}

fn rust_gethostname() -> Result<String, ()> {
    let len = 255;
    let mut buf = Vec::<u8>::with_capacity(len);

    let ptr = buf.as_mut_slice().as_mut_ptr();

    let err = unsafe { gethostname(ptr as *mut libc::c_char, len as libc::size_t) } as isize;

    match err {
        0 => {
            let mut real_len = len;
            let mut i = 0;
            loop {
                if i >= len {
                    break;
                }
                let byte = unsafe { *(((ptr as u64) + (i as u64)) as *const u8) };
                if byte == 0 {
                    real_len = i;
                    break;
                }

                i += 1;
            }
            unsafe { buf.set_len(real_len) }
            Ok(String::from_utf8_lossy(buf.as_ref()).into_owned())
        }
        _ => Err(()),
    }
}

/// Gives access to the next middleware for a command.
pub struct NextMiddleware<CT, ST> {
    callback: MiddlewareFn<CT, ST>,
    next: Box<Option<NextMiddleware<CT, ST>>>,
}

impl<CT, ST> Clone for NextMiddleware<CT, ST> {
    fn clone(&self) -> NextMiddleware<CT, ST> {
        NextMiddleware {
            callback: self.callback,
            next: self.next.clone(),
        }
    }
}

impl<CT, ST> NextMiddleware<CT, ST> {
    /// Call a command middleware.
    pub fn call(
        &self,
        config: &ServerConfig<CT>,
        container: &mut CT,
        i: &mut InputStream<ST>,
        o: &mut OutputStream<ST>,
        l: &str,
    ) {
        match *self.next {
            Some(ref next) => {
                (self.callback)(config, container, i, o, l, Some(next.clone()));
            }
            None => {
                (self.callback)(config, container, i, o, l, None);
            }
        }
    }
}

/// A command middleware callback.
pub type MiddlewareFn<CT, ST> = fn(
    &ServerConfig<CT>,
    &mut CT,
    &mut InputStream<ST>,
    &mut OutputStream<ST>,
    &str,
    Option<NextMiddleware<CT, ST>>,
) -> ();

/// An email server command.
///
/// It is defined by the string you find at the start of the command, for
/// example "MAIL FROM:" or "EHLO ", as well as a bunch of middleware parts
/// that are executed sequentially until one says to stop.
pub struct Command<CT, ST> {
    start: Option<String>,
    front_middleware: Option<NextMiddleware<CT, ST>>,
}

impl<CT, ST> Clone for Command<CT, ST> {
    fn clone(&self) -> Command<CT, ST> {
        Command {
            start: self.start.clone(),
            front_middleware: self.front_middleware.clone(),
        }
    }
}

impl<CT, ST> Command<CT, ST> {
    /// Creates a new command
    pub fn new() -> Command<CT, ST> {
        Command {
            start: None,
            front_middleware: None,
        }
    }

    /// Describes the start of the command line for this command.
    pub fn starts_with(&mut self, start: &str) {
        self.start = Some(start.to_owned());
    }

    fn last_middleware<'a>(prev: &'a mut NextMiddleware<CT, ST>) -> &'a mut NextMiddleware<CT, ST> {
        match *prev.next {
            None => prev,
            Some(ref mut next) => Command::last_middleware(next),
        }
    }

    /// Add a middleware to call for this command.
    pub fn middleware(&mut self, callback: MiddlewareFn<CT, ST>) {
        // The upcoming item in the middleware chain.
        let next = Some(NextMiddleware {
            callback: callback,
            next: Box::new(None),
        });

        // Get the current last item, so we can append the new item.
        match self.front_middleware {
            None => {
                self.front_middleware = next;
            }
            Some(_) => {
                Command::last_middleware(self.front_middleware.as_mut().unwrap()).next =
                    Box::new(next);
            }
        }
    }
}

/// An SMTP server configuration.
pub struct ServerConfig<CT> {
    hostname: String,
    max_recipients: usize,
    max_message_size: usize,
    max_command_line_size: usize,
    max_text_line_size: usize,
    commands: Vec<Command<CT, TcpStream>>,
    extensions: Vec<String>,
}

impl<CT> Clone for ServerConfig<CT> {
    fn clone(&self) -> ServerConfig<CT> {
        // TcpStream is non clonable, which seems to disturb the compiler, so we clone
        // the commands vector (which is made of commands that take a TcpStream) manually.
        let mut cloned_commands = Vec::with_capacity(self.commands.len());
        for c in self.commands.iter() {
            cloned_commands.push(c.clone());
        }

        ServerConfig {
            hostname: self.hostname.clone(),
            max_recipients: self.max_recipients,
            max_message_size: self.max_message_size,
            max_command_line_size: self.max_command_line_size,
            max_text_line_size: self.max_text_line_size,
            commands: cloned_commands,
            extensions: self.extensions.clone(),
        }
    }
}

/// An SMTP server, with no commands by default.
pub struct Server<CT> {
    config: ServerConfig<CT>,
    container: CT,
}

/// An error that occures when a server starts up
#[derive(PartialEq, Eq, Clone, Debug, Copy)]
pub enum ServerError {
    /// The hostname could not be retrieved from the system
    Hostname,
    /// Could not bind the socket
    Bind,
    /// Could not listen on the socket
    Listen,
}

/// Tells whether an error occured during server setup.
pub type ServerResult<T> = Result<T, ServerError>;

impl<CT: 'static + Send + Sync + Clone> Server<CT> {
    /// Creates a new SMTP server.
    ///
    /// The container can be of any type and can be used to get access to a
    /// bunch of things inside your commands, like database connections,
    /// a logger and more.
    pub fn new(container: CT) -> Server<CT> {
        Server {
            config: ServerConfig {
                hostname: String::new(),
                max_recipients: 100,
                max_message_size: 65536,
                max_command_line_size: 512,
                max_text_line_size: 1000,
                commands: Vec::with_capacity(16),
                extensions: Vec::with_capacity(16),
            },
            container: container,
        }
    }

    pub fn set_hostname(&mut self, hostname: &str) {
        self.config.hostname = hostname.to_owned();
    }

    pub fn set_max_recipients(&mut self, max: usize) {
        if max < 100 {
            panic!("Maximum number of recipients must be >= 100.");
        }
        self.config.max_recipients = max;
    }

    pub fn set_max_message_size(&mut self, max: usize) {
        if max < 65536 {
            panic!("Maximum message size must be >= 65536.");
        }
        self.config.max_message_size = max;
    }

    /// Adds a command to the server.
    pub fn add_command(&mut self, command: Command<CT, TcpStream>) {
        self.config.commands.push(command);
    }

    pub fn increase_max_command_line_size(&mut self, bytes: usize) {
        self.config.max_command_line_size += bytes;
    }

    pub fn increase_max_text_line_size(&mut self, bytes: usize) {
        self.config.max_text_line_size += bytes;
    }

    /// Marks an SMTP extension as "supported" by the server.
    ///
    /// This is used in the output of the EHLO command.
    pub fn add_extension(&mut self, extension: &str) {
        self.config.extensions.push(extension.to_owned());
    }

    fn get_hostname_from_system(&mut self) -> ServerResult<String> {
        match rust_gethostname() {
            Ok(s) => Ok(s),
            Err(_) => Err(ServerError::Hostname),
        }
    }

    fn get_listener_for_address(&mut self, address: (IpAddr, u16)) -> ServerResult<TcpListener> {
        match TcpListener::bind(address) {
            Ok(listener) => Ok(listener),
            Err(_) => Err(ServerError::Bind),
        }
    }

    fn handle_commands(
        config: &ServerConfig<CT>,
        input: &mut InputStream<TcpStream>,
        output: &mut OutputStream<TcpStream>,
        container: &mut CT,
    ) {
        'main: loop {
            let line = match input.read_line() {
                Ok(buffer) => {
                    // The commands expect a regular human readable string.
                    // Also, we need to make this an owned string because
                    // the stream uses the same buffer for command lines and
                    // text lines.
                    String::from_utf8_lossy(buffer).into_owned()
                }
                Err(err) => {
                    panic!("Could not read command: {}", err);
                }
            };

            // Find the right handler for this command line.
            for command in config.commands.iter() {
                // The right command starts with whatever we have set
                // when we created the command. We use unwrap here, but
                // the commands are checked before the server starts
                // so this is always OK.
                match command.start {
                    Some(ref start) => {
                        let ls = line.as_str();
                        // TODO: make this case insensitive
                        if ls.starts_with(start.as_str()) {
                            match command.front_middleware {
                                Some(ref next) => {
                                    next.call(config, container, input, output, &ls[start.len()..]);
                                }
                                None => {
                                    panic!("Found a command with no middleware");
                                }
                            }
                            continue 'main;
                        }
                    }
                    None => {
                        panic!("Found a command with no start string");
                    }
                }
            }

            // If we get here, it means that no command matched.
            output.write_line("500 Command unrecognized").unwrap();
        }
    }

    fn handle_connection(&self, stream_res: IoResult<TcpStream>, config: &Arc<ServerConfig<CT>>) {
        let config = config.clone();
        let mut container = self.container.clone();
        let thread_handle = thread::spawn(move || {
            match stream_res {
                Ok(stream) => {
                    // Clone the stream. This uses "unsafe" but is safe because we use this
                    // stream only for reading and the other one only for writing.
                    let mut input = InputStream::new(
                        unsafe { TcpStream::from_raw_fd(stream.as_raw_fd()) },
                        1000,
                        false,
                    );
                    let mut output = OutputStream::new(stream, false);

                    Server::<CT>::handle_commands(
                        config.deref(),
                        &mut input,
                        &mut output,
                        &mut container,
                    );
                }
                Err(err) => {
                    panic!("Could not accept client: {}", err);
                }
            }
        });
        println!(
            "Connection being handled in thread: {:?}",
            thread_handle.thread().name()
        );
    }

    /// Start the SMTP server on the given address and port.
    pub fn listen(&mut self, ip: IpAddr, port: u16) -> ServerResult<()> {
        if self.config.hostname.len() == 0 {
            self.config.hostname = self.get_hostname_from_system()?;
        }

        let listener = self.get_listener_for_address((ip, port))?;

        println!(
            "Server '{}' listening on {}:{}...",
            self.config.hostname, ip, port
        );

        let config = Arc::new(self.config.clone());

        for conn in listener.incoming() {
            self.handle_connection(conn, &config);
        }

        Ok(())
    }
}

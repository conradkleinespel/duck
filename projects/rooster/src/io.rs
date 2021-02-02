use ansi_term::Color::{Green, Red, Yellow};
pub use ansi_term::Colour;
use ansi_term::Style as AnsiTermStyle;
use rpassword::{read_password_from_bufread, read_password_from_tty};
use safe_string::SafeString;
use std::fs::File;
use std::io::{BufRead, Result as IoResult};
use std::io::{Cursor, StderrLock, StdinLock, StdoutLock, Write};

pub enum OutputType {
    Standard,
    Error,
    Tty,
}

/// Input that reads from stdin, useful for regular CLI use
pub struct RegularInput<'a> {
    pub stdin_lock: StdinLock<'a>,
}

/// Output for writing to stdout/stderr, useful for regular CLI use
pub struct RegularOutput<'a> {
    pub stdout_lock: StdoutLock<'a>,
    pub stderr_lock: StderrLock<'a>,
    pub tty: File,
}

/// Input that reads from a cursor, useful for tests
#[derive(Default)]
pub struct CursorInput {
    cursor: Cursor<Vec<u8>>,
}

impl CursorInput {
    pub fn new(input: &str) -> CursorInput {
        CursorInput {
            cursor: Cursor::new(input.as_bytes().to_owned()),
        }
    }
}

/// Output for writing to cursors, useful for tests
#[derive(Default)]
pub struct CursorOutput {
    pub standard_cursor: Cursor<Vec<u8>>,
    pub error_cursor: Cursor<Vec<u8>>,
    pub tty_cursor: Cursor<Vec<u8>>,
}

impl CursorOutput {
    pub fn new() -> CursorOutput {
        CursorOutput {
            standard_cursor: Cursor::new(Vec::new()),
            error_cursor: Cursor::new(Vec::new()),
            tty_cursor: Cursor::new(Vec::new()),
        }
    }
}

pub trait CliReader {
    fn read_line(&mut self) -> IoResult<String>;
    fn read_password(&mut self) -> IoResult<SafeString>;
}

pub trait CliWriter {
    fn nl(&mut self, output_type: OutputType);
    fn write(&mut self, s: impl ToString, output_type: OutputType);
    fn writeln(&mut self, s: impl ToString, output_type: OutputType);
}

#[derive(Clone)]
pub struct Style;

impl Style {
    pub fn title(s: impl ToString) -> String {
        AnsiTermStyle::new()
            .underline()
            .bold()
            .paint(s.to_string())
            .to_string()
    }

    pub fn info(s: impl ToString) -> String {
        AnsiTermStyle::new().paint(s.to_string()).to_string()
    }

    pub fn warning(s: impl ToString) -> String {
        Yellow.normal().paint(s.to_string()).to_string()
    }

    pub fn error(s: impl ToString) -> String {
        Red.normal().paint(s.to_string()).to_string()
    }

    pub fn success(s: impl ToString) -> String {
        Green.normal().paint(s.to_string()).to_string()
    }
}

impl<'a> CliReader for RegularInput<'a> {
    fn read_line(&mut self) -> IoResult<String> {
        let mut s = String::new();
        self.stdin_lock.read_line(&mut s)?;
        Ok(s)
    }

    fn read_password(&mut self) -> IoResult<SafeString> {
        Ok(SafeString::new(read_password_from_tty()?))
    }
}

impl<'a> CliWriter for RegularOutput<'a> {
    fn nl(&mut self, output_type: OutputType) {
        match output_type {
            OutputType::Standard => {
                self.stdout_lock.write_all("\n".as_bytes()).unwrap();
                self.stdout_lock.flush().unwrap();
            }
            OutputType::Error => {
                self.stderr_lock.write_all("\n".as_bytes()).unwrap();
                self.stderr_lock.flush().unwrap();
            }
            OutputType::Tty => {
                self.tty.write_all("\n".as_bytes()).unwrap();
                self.tty.flush().unwrap();
            }
        }
    }

    fn write(&mut self, s: impl ToString, output_type: OutputType) {
        match output_type {
            OutputType::Standard => {
                self.stdout_lock
                    .write_all(s.to_string().as_bytes())
                    .unwrap();
                self.stdout_lock.flush().unwrap();
            }
            OutputType::Error => {
                self.stderr_lock
                    .write_all(s.to_string().as_bytes())
                    .unwrap();
                self.stderr_lock.flush().unwrap();
            }
            OutputType::Tty => {
                self.tty.write_all(s.to_string().as_bytes()).unwrap();
                self.tty.flush().unwrap();
            }
        }
    }

    fn writeln(&mut self, s: impl ToString, output_type: OutputType) {
        match output_type {
            OutputType::Standard => {
                self.stdout_lock
                    .write_all(s.to_string().as_bytes())
                    .unwrap();
                self.stdout_lock.write_all("\n".as_bytes()).unwrap();
                self.stdout_lock.flush().unwrap();
            }
            OutputType::Error => {
                self.stderr_lock
                    .write_all(s.to_string().as_bytes())
                    .unwrap();
                self.stderr_lock.write_all("\n".as_bytes()).unwrap();
                self.stderr_lock.flush().unwrap();
            }
            OutputType::Tty => {
                self.tty.write_all(s.to_string().as_bytes()).unwrap();
                self.tty.write_all("\n".as_bytes()).unwrap();
                self.tty.flush().unwrap();
            }
        }
    }
}

impl CliReader for CursorInput {
    fn read_line(&mut self) -> IoResult<String> {
        let mut s = String::new();
        self.cursor.read_line(&mut s)?;
        Ok(s)
    }

    fn read_password(&mut self) -> IoResult<SafeString> {
        Ok(SafeString::new(read_password_from_bufread(
            &mut self.cursor,
        )?))
    }
}

impl CliWriter for CursorOutput {
    fn nl(&mut self, output_type: OutputType) {
        match output_type {
            OutputType::Standard => {
                self.standard_cursor.write_all("\n".as_bytes()).unwrap();
                self.standard_cursor.flush().unwrap();
            }
            OutputType::Error => {
                self.error_cursor.write_all("\n".as_bytes()).unwrap();
                self.error_cursor.flush().unwrap();
            }
            OutputType::Tty => {
                self.tty_cursor.write_all("\n".as_bytes()).unwrap();
                self.tty_cursor.flush().unwrap();
            }
        }
    }

    fn write(&mut self, s: impl ToString, output_type: OutputType) {
        match output_type {
            OutputType::Standard => {
                self.standard_cursor
                    .write_all(s.to_string().as_bytes())
                    .unwrap();
                self.standard_cursor.flush().unwrap();
            }
            OutputType::Error => {
                self.error_cursor
                    .write_all(s.to_string().as_bytes())
                    .unwrap();
                self.error_cursor.flush().unwrap();
            }
            OutputType::Tty => {
                self.tty_cursor.write_all(s.to_string().as_bytes()).unwrap();
                self.tty_cursor.flush().unwrap();
            }
        }
    }

    fn writeln(&mut self, s: impl ToString, output_type: OutputType) {
        match output_type {
            OutputType::Standard => {
                self.standard_cursor
                    .write_all(s.to_string().as_bytes())
                    .unwrap();
                self.standard_cursor
                    .write_all("\n".to_string().as_bytes())
                    .unwrap();
                self.standard_cursor.flush().unwrap();
            }
            OutputType::Error => {
                self.error_cursor
                    .write_all(s.to_string().as_bytes())
                    .unwrap();
                self.error_cursor
                    .write_all("\n".to_string().as_bytes())
                    .unwrap();
                self.error_cursor.flush().unwrap();
            }
            OutputType::Tty => {
                self.tty_cursor.write_all(s.to_string().as_bytes()).unwrap();
                self.tty_cursor
                    .write_all("\n".to_string().as_bytes())
                    .unwrap();
                self.tty_cursor.flush().unwrap();
            }
        }
    }
}

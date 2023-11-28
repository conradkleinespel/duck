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

//! Tools for reading/writing from SMTP clients to SMTP servers and vice-versa.

#[cfg(test)]
use super::MIN_ALLOWED_LINE_SIZE;
#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::fs::OpenOptions;
use std::io::Error as IoError;
use std::io::Result as IoResult;
use std::io::{ErrorKind, Read, Write};
#[cfg(test)]
use std::iter::{repeat, FromIterator};
use std::ops::{IndexMut, RangeFrom};
use std::vec::Vec;

pub static LINE_TOO_LONG: &str = "line too long";

pub static DATA_TOO_LONG: &str = "message too long";

#[test]
fn test_static_vars() {
    // Already tested in the limits test further down.
}

/// A stream that reads lines of input.
///
/// # Example
/// ```no_run
/// use std::net::TcpStream;
/// use rsmtp::common::stream::InputStream;
/// use rsmtp::common::{
///     MIN_ALLOWED_LINE_SIZE,
/// };
///
/// let mut smtp = InputStream::new(
///     TcpStream::connect("127.0.0.1:25").unwrap(),
///     MIN_ALLOWED_LINE_SIZE,
///     false
/// );
///
/// println!("{:?}", smtp.read_line().unwrap());
/// ```
pub struct InputStream<S> {
    /// Underlying stream
    stream: S,
    /// Must be at least 1001 per RFC 5321, 1000 chars + 1 for transparency
    /// mechanism.
    max_line_size: usize,
    /// Buffer to make reading more efficient and allow pipelining
    buf: Vec<u8>,
    /// If `true`, will print debug messages of input and output to the console.
    debug: bool,
    /// The position of the `<CRLF>` found at the previous `read_line`.
    last_crlf: Option<usize>,
}

// The state of the `<CRLF>` search inside a buffer. See below.
enum CRLFState {
    // We are looking for `<CR>`.
    Cr,
    // We are looking for `<LF>`.
    Lf,
}

// Find the position of the first `<CRLF>` in a buffer.
fn position_crlf(buf: &[u8]) -> Option<usize> {
    let mut state = CRLFState::Cr;
    let mut index = 0;

    for byte in buf.iter() {
        match state {
            CRLFState::Cr => {
                if byte == &13 {
                    state = CRLFState::Lf;
                }
            }
            CRLFState::Lf => {
                if byte == &10 {
                    // Subtract 1 to account for the \r, seen previously.
                    return Some(index - 1);
                }
            }
        }
        index += 1;
    }

    None
}

impl<S: Read> InputStream<S> {
    /// Create a new `InputStream` from another stream.
    pub fn new(inner: S, max_line_size: usize, debug: bool) -> InputStream<S> {
        InputStream {
            stream: inner,
            max_line_size: max_line_size,
            // TODO: make line reading work even with a buffer smaller than the maximum line size.
            // Currently, this will not work because we only fill the buffer once per line, assuming
            // that the buffer is large enough.
            buf: Vec::with_capacity(max_line_size),
            debug: debug,
            last_crlf: None,
        }
    }

    /// Remove the previous line from the buffer when reading a new line.
    pub fn move_buf(&mut self) {
        // Remove the last line, since we've used it already by now.
        match self.last_crlf {
            Some(p) => {
                self.buf = self.buf[p + 2..].to_vec();
                self.buf.reserve(self.max_line_size);
            }
            _ => {}
        }

        self.last_crlf = None;
    }

    /// Fill the buffer to its limit.
    fn fill_buf(&mut self) -> IoResult<usize> {
        let len = self.buf.len();
        let cap = self.buf.capacity();

        // Leave as much space open at the end of the buffer so we can fill it using
        // a &mut reference. We'll set the right length again later.
        unsafe { self.buf.set_len(cap) };

        // Read as much data as the buffer can hold without re-allocation.
        match self
            .stream
            .read(self.buf.index_mut(RangeFrom { start: len }))
        {
            Ok(num_bytes) => {
                // Set the new known length for the buffer.
                unsafe { self.buf.set_len(len + num_bytes) };
                Ok(num_bytes)
            }
            Err(err) => {
                // Restore the previous length so we don't accidentally use outdated bytes.
                unsafe { self.buf.set_len(len) };
                Err(err)
            }
        }
    }

    /// Read an SMTP command. Ends with `<CRLF>`.
    pub fn read_line(&mut self) -> IoResult<&[u8]> {
        // Remove the previous line from the buffer before reading a new one.
        self.move_buf();

        let read_line = match position_crlf(self.buf.as_ref()) {
            // First, let's check if the buffer already contains a line. This
            // reduces the number of syscalls.
            Some(last_crlf) => {
                self.last_crlf = Some(last_crlf);
                Ok(&self.buf[..last_crlf])
            }
            // If we don't have a line in the buffer, we'll read more input
            // and try again.
            None => {
                match self.fill_buf() {
                    Ok(_) => {
                        match position_crlf(self.buf.as_ref()) {
                            Some(last_crlf) => {
                                self.last_crlf = Some(last_crlf);
                                Ok(&self.buf[..last_crlf])
                            }
                            None => {
                                // If we didn't find a line, it means we had
                                // no `<CRLF>` in the buffer, which means that
                                // the line is too long.
                                Err(IoError::new(ErrorKind::InvalidInput, LINE_TOO_LONG))
                            }
                        }
                    }
                    Err(err) => Err(err),
                }
            }
        };

        // If we read a line, we'll say so in the console, if debug mode is on.
        if let Ok(bytes) = read_line {
            if self.debug {
                println!("rsmtp: imsg: {}", String::from_utf8_lossy(bytes.as_ref()));
            }
        }

        read_line
    }
}

/// A stream that writes lines of output.
pub struct OutputStream<S> {
    /// Underlying stream
    stream: S,
    /// If `true`, will print debug messages of input and output to the console.
    debug: bool,
}

impl<S: Write> OutputStream<S> {
    /// Create a new `InputStream` from another stream.
    pub fn new(inner: S, debug: bool) -> OutputStream<S> {
        OutputStream {
            stream: inner,
            debug: debug,
        }
    }

    /// Write a line ended with `<CRLF>`.
    pub fn write_line(&mut self, s: &str) -> IoResult<()> {
        if self.debug {
            println!("rsmtp: omsg: {}", s);
        }
        // We use `format!()` instead of 2 calls to `write_str()` to reduce
        // the amount of syscalls and to send the string as a single packet.
        // I'm not sure if this is the right way to go though. If you think
        // this is wrong, please open a issue on Github.
        write!(&mut self.stream, "{}\r\n", s)
    }
}

#[test]
fn test_new() {
    // This method is already tested via `test_read_line()`.
}

#[test]
fn test_write_line() {
    // Use a block so the file gets closed at the end of it.
    {
        let file_write: File;
        let mut stream: OutputStream<File>;

        file_write = OpenOptions::new()
            .truncate(true)
            .write(true)
            .open("tests/stream/write_line")
            .unwrap();
        stream = OutputStream::new(file_write, false);
        stream.write_line("HelloWorld").unwrap();
        stream.write_line("ByeBye").unwrap();
    }
    let mut file_read: File;
    let mut expected = String::new();

    file_read = OpenOptions::new()
        .read(true)
        .open("tests/stream/write_line")
        .unwrap();
    file_read.read_to_string(&mut expected).unwrap();
    assert_eq!("HelloWorld\r\nByeBye\r\n", expected.as_str());
}

#[test]
fn test_limits() {
    let file: File;
    let mut stream: InputStream<File>;

    file = OpenOptions::new()
        .read(true)
        .open("tests/stream/1line1")
        .unwrap();
    stream = InputStream::new(file, 3, false);
    match stream.read_line() {
        Ok(_) => panic!(),
        Err(err) => {
            assert_eq!("line too long", err.to_string());
            assert_eq!(ErrorKind::InvalidInput, err.kind());
        }
    }
}

#[test]
fn test_read_line() {
    let mut file: File;
    let mut stream: InputStream<File>;
    let expected: String;

    file = OpenOptions::new()
        .read(true)
        .open("tests/stream/0line1")
        .unwrap();
    stream = InputStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert!(!stream.read_line().is_ok());

    file = OpenOptions::new()
        .read(true)
        .open("tests/stream/0line2")
        .unwrap();
    stream = InputStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert!(!stream.read_line().is_ok());

    file = OpenOptions::new()
        .read(true)
        .open("tests/stream/0line3")
        .unwrap();
    stream = InputStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert!(!stream.read_line().is_ok());

    file = OpenOptions::new()
        .read(true)
        .open("tests/stream/1line1")
        .unwrap();
    stream = InputStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref())
            .to_owned()
            .as_ref(),
        "hello world!"
    );
    assert!(!stream.read_line().is_ok());

    file = OpenOptions::new()
        .read(true)
        .open("tests/stream/1line2")
        .unwrap();
    stream = InputStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref())
            .to_owned()
            .as_ref(),
        "hello world!"
    );
    assert!(!stream.read_line().is_ok());

    file = OpenOptions::new()
        .read(true)
        .open("tests/stream/2lines1")
        .unwrap();
    stream = InputStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref())
            .to_owned()
            .as_ref(),
        "hello world!"
    );
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref())
            .to_owned()
            .as_ref(),
        "bye bye world!"
    );
    assert!(!stream.read_line().is_ok());

    expected = String::from_iter(repeat('x').take(62));
    file = OpenOptions::new()
        .read(true)
        .open("tests/stream/xlines1")
        .unwrap();
    stream = InputStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref()).to_owned(),
        expected
    );
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref()).to_owned(),
        expected
    );
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref()).to_owned(),
        expected
    );
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref()).to_owned(),
        expected
    );
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref()).to_owned(),
        expected
    );
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref()).to_owned(),
        expected
    );
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref()).to_owned(),
        expected
    );
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref()).to_owned(),
        expected
    );
    assert_eq!(
        String::from_utf8_lossy(stream.read_line().unwrap().as_ref()).to_owned(),
        expected
    );
    assert!(!stream.read_line().is_ok());
}

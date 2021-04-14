use std::fs::read_dir;
use std::io;

pub fn validate_dir(path: &str) -> io::Result<()> {
    read_dir(path).map(|_| ())
}

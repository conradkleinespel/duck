pub mod prelude {
    extern crate rooster;
    extern crate tempfile;

    pub use self::rooster::io::{CursorInput, CursorOutput};
    pub use self::rooster::main_with_args;
    pub fn tempfile() -> PathBuf {
        self::tempfile::NamedTempFile::new()
            .unwrap()
            .path()
            .to_path_buf()
    }
    pub use std::io::Cursor;
    use std::path::PathBuf;
}

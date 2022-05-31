use std::{error, io};

use simple_error::SimpleError;

pub fn other_io_error(e: impl Into<Box<dyn error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

pub fn string_io_error(s: String) -> io::Error {
    other_io_error(SimpleError::new(s))
}

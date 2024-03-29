use std;
use std::fmt::{self, Display};
use std::io;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Custom(String),
    /// Trailing character after the data
    TrailingCharacters,
    Nom(nom::ErrorKind),
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(std::error::Error::description(self))
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Custom(ref msg) => msg,
            Error::TrailingCharacters => "unexpected trailing characters after the data",
            Error::Nom(ref e) => e.description(),
            Error::Io(ref e) => e.description(),
        }
    }
}

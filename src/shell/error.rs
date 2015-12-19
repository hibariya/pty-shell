use pty;
use std::{self, io, fmt};

#[derive(Debug)]
pub enum Error {
    Pty(pty::Error),
    Io(io::Error),
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "pty-shell error"
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Pty(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        std::error::Error::description(self).fmt(f)
    }
}

impl From<pty::Error> for Error {
    fn from(err: pty::Error) -> Error {
        Error::Pty(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

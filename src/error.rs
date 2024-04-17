pub enum Error {
    IoError(std::io::Error),
    ParseError(rsip::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<rsip::Error> for Error {
    fn from(err: rsip::Error) -> Error {
        Error::ParseError(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(e) => e.fmt(f),
            Error::ParseError(e) => e.fmt(f),
        }
    }
}

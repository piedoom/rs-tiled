use std::fmt;
use xml::reader::Error as XmlError;

/// Errors which occured when parsing the file
#[derive(Debug)]
pub enum Error {
    /// A attribute was missing, had the wrong type of wasn't formated
    /// correctly.
    MalformedAttributes(String),
    /// An error occured when decompressing using the
    /// [flate2](https://github.com/alexcrichton/flate2-rs) crate.
    DecompressingError(std::io::Error),
    Base64DecodingError(base64::DecodeError),
    XmlDecodingError(XmlError),
    PrematureEnd(String),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::MalformedAttributes(ref s) => write!(fmt, "{}", s),
            Error::DecompressingError(ref e) => write!(fmt, "{}", e),
            Error::Base64DecodingError(ref e) => write!(fmt, "{}", e),
            Error::XmlDecodingError(ref e) => write!(fmt, "{}", e),
            Error::PrematureEnd(ref e) => write!(fmt, "{}", e),
            Error::Other(ref s) => write!(fmt, "{}", s),
        }
    }
}

// This is a skeleton implementation, which should probably be extended in the future.
impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::MalformedAttributes(ref s) => s.as_ref(),
            Error::DecompressingError(ref e) => e.description(),
            Error::Base64DecodingError(ref e) => e.description(),
            Error::XmlDecodingError(ref e) => e.description(),
            Error::PrematureEnd(ref s) => s.as_ref(),
            Error::Other(ref s) => s.as_ref(),
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::MalformedAttributes(_) => None,
            Error::DecompressingError(ref e) => Some(e as &std::error::Error),
            Error::Base64DecodingError(ref e) => Some(e as &std::error::Error),
            Error::XmlDecodingError(ref e) => Some(e as &std::error::Error),
            Error::PrematureEnd(_) => None,
            Error::Other(_) => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ParseTileError {
    ColorError,
    OrientationError,
}

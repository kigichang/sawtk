use std::{error, fmt, result};
use protobuf::ProtobufError;
use sawtooth_sdk::signing::Error as SignErr;

// utility for dataforce.
pub mod util;
pub mod wallet;

// sawtooth toolkit
pub mod ns;
pub mod tx;
pub mod tp;
pub mod signing;
pub mod messages;

// ----------------------------------------------------------------------------

#[derive(Debug)]
pub enum Error {
    InvalidChar(usize, char),
    OddLengthString(usize),
    Protobuf(ProtobufError),
    Signing(SignErr),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Error::InvalidChar(idx, ch) => write!(f, "invalid hex character {} at {}", ch, idx),
            Error::OddLengthString(len) => write!(f, "odd hex string length {}", len),
            Error::Protobuf(e) => write!(f, "encode/decode proto message {}", e),
            Error::Signing(e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match &self {
            Error::Protobuf(ref e) => Some(e),
            Error::Signing(ref e) => Some(e),
            _ => None,
        }
    }

    fn description(&self) -> &'static str {
        match *self {
            Error::InvalidChar(_, _) => "invalid hex character",
            Error::OddLengthString(_) => "odd hex string length",
            Error::Protobuf(_) => "encode/decode proto message failure",
            Error::Signing(_) => "signing error",
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
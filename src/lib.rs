/*
 * MIT License
 *
 * Copyright (c) 2019 Kigi Chang

 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.

 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
*/

extern crate bs58;
extern crate sha2;
extern crate uuid;

//use bs58::decode::Error as bs58dErr;
use protobuf::ProtobufError;
use sawtooth_sdk::signing::Error as SignErr;
use std::{error, fmt, result};

// utility for dataforce.
pub mod util;
pub mod wallet;

// sawtooth toolkit
pub mod messages;
pub mod ns;
pub mod signing;
pub mod tp;
pub mod tx;

// ----------------------------------------------------------------------------

#[derive(Debug)]
pub enum Error {
    InvalidChar(usize, char),
    OddLengthString(usize),
    Protobuf(ProtobufError),
    Signing(SignErr),
    //InvalidLength(usize),
    //InvalidVersion(u8, u8),
    //BS58(bs58dErr),
    //CheckSum,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Error::InvalidChar(idx, ch) => write!(f, "invalid hex character {} at {}", ch, idx),
            Error::OddLengthString(len) => write!(f, "odd hex string length {}", len),
            Error::Protobuf(e) => write!(f, "encode/decode proto message {}", e),
            Error::Signing(e) => e.fmt(f),
            /*Error::InvalidLength(len) => write!(f, "invalid length {}", len),
            Error::InvalidVersion(test, ans) => {
                write!(f, "invalid version {}, must be {}", test, ans)
            }
            Error::BS58(e) => write!(f, "base58 decode: {:?}", e),
            Error::CheckSum => write!(f, "checksum not match"),*/
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match &self {
            Error::Protobuf(ref e) => Some(e),
            Error::Signing(ref e) => Some(e),
            //Error::BS58(ref e) => Some(e),
            _ => None,
        }
    }

    fn description(&self) -> &'static str {
        match *self {
            Error::InvalidChar(_, _) => "invalid hex character",
            Error::OddLengthString(_) => "odd hex string length",
            Error::Protobuf(_) => "encode/decode proto message failure",
            Error::Signing(_) => "signing error",
            /*Error::InvalidLength(_) => "invalid length",
            Error::InvalidVersion(_, _) => "invalid version",
            Error::BS58(_) => "base58 decode failure",
            Error::CheckSum => "checksum not match",*/
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

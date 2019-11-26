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

use sha2::{Sha256, Sha512, Digest};
use uuid::Uuid;
use super::{Result, Error};

//-----------------------------------------------------------------------------

/*
#[derive(Debug, Copy, Clone)]
pub enum Error {
    InvalidChar(usize, char),
    OddLengthString(usize),
}
*/


//------------------------------------------------------------------------------

/// Converts bytes to hex string.
pub fn bytes_to_hex_str(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<_>>().join("")
}

// Converts hex string to bytes.
pub fn hex_str_to_bytes(hexstr: &str) -> Result<Vec<u8>> {
    let _ = is_hex(hexstr)?;

    let tmp = hexstr.chars().collect::<Vec<_>>();

    let ret: Vec<u8> = tmp.chunks(2)
        .map(|c| {
            ((c[0].to_digit(16).unwrap() << 4) | (c[1].to_digit(16).unwrap())) as u8
        }).collect();

    Ok(ret)
}

pub fn is_hex_str(test: &str) -> bool {
    is_hex(test).is_ok()
}

fn is_hex(hexstr: &str) -> Result<()> {
    let size = hexstr.len();

    if size % 2 != 0 {
        return Err(Error::OddLengthString(size));
    }

    for (idx, ch) in hexstr.chars().enumerate() {
        if !ch.is_digit(16) {
            return Err(Error::InvalidChar(idx, ch));
        }
    }

    Ok(())
}
//------------------------------------------------------------------------------

pub fn is_public_key(key: &str) -> bool {
    if key.len() != 66 {
        return false;
    }

    is_hex_str(key)
}

//------------------------------------------------------------------------------

pub fn sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.input(input);
    bytes_to_hex_str(&hasher.result())
}


pub fn sha256_raw(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.input(input);
    Vec::from(hasher.result().as_slice())
}

pub fn sha512_raw(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.input(input);
    Vec::from(hasher.result().as_slice())
}

//------------------------------------------------------------------------------

pub fn sha512(input: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.input(input);
    bytes_to_hex_str(&hasher.result())
}

pub fn sha512_bytes(input: &[u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.input(input);
    bytes_to_hex_str(&hasher.result())
}

//------------------------------------------------------------------------------

pub fn uuid() -> String {
    Uuid::new_v4().to_hyphenated().to_string()
}

pub fn is_uuid(_test: &str) -> bool {
    // TODO: verify uuid string.
    true
}
//------------------------------------------------------------------------------

pub fn nonce() -> String {
    uuid()
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        assert_eq!("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9", 
        sha256("hello world"))
    }

    #[test]
    fn test_sha512() {
        assert_eq!("309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f",
        sha512("hello world"))
    }

    #[test]
    fn test_uuid() {
        println!("{}", uuid());
    }
}


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

// TODO: re-org, use version is 0x00;

use super::util::{bytes_to_hex_str, hex_str_to_bytes, sha256_raw};
use super::Error;
use super::Result;
use bs58;

// ----------------------------------------------------------------------------

const CHECKSUM_LEN: usize = 4;

fn checksum(input: &[u8]) -> Vec<u8> {
    let h = sha256_raw(input);
    let h2 = sha256_raw(&h);

    Vec::from(&h2[..CHECKSUM_LEN])
}

fn check_encode(input: &[u8], version: u8) -> String {
    let mut b = Vec::with_capacity(1 + input.len() + CHECKSUM_LEN);

    b.push(version);
    b.extend_from_slice(input);

    let cksum = checksum(&b);

    b.extend_from_slice(&cksum);
    bs58::encode(&b).into_string()
}

fn check_decode(decoded: &[u8]) -> Result<(String, u8)> {
    let decoded_len = decoded.len();

    if decoded_len < (1 + CHECKSUM_LEN) {
        return Err(Error::InvalidLength(decoded_len));
    }

    let version = decoded[0];
    let sum1: &[u8] = &decoded[decoded_len - CHECKSUM_LEN..];
    let sum2 = checksum(&decoded[0..(decoded_len - CHECKSUM_LEN)]);

    if sum1[0] != sum2[0] || sum1[1] != sum2[1] || sum1[2] != sum2[2] || sum1[3] != sum2[3] {
        return Err(Error::CheckSum);
    }

    if version != decoded[1] {
        return Err(Error::InvalidVersion(version, decoded[1]));
    }

    Ok((
        bytes_to_hex_str(&decoded[1..(decoded_len - CHECKSUM_LEN)]),
        version,
    ))
}

pub fn new(input: &[u8]) -> String {
    check_encode(input, input[0])
}

pub fn from_hex(hexstr: &str) -> Result<String> {
    hex_str_to_bytes(hexstr).map(|x| new(&x))
}

pub fn to_public_key(wallet: &str) -> Result<(String, u8)> {
    let input = bs58::decode(wallet)
        .into_vec()
        .map_err(|e| Error::BS58(e))?;
    check_decode(&input)
}

pub fn is_wallet(wallet: &str) -> bool {
    to_public_key(wallet).is_ok()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from() {
        let result =
            from_hex("03511c83916ac338835b07f6b9f7c0aa10b7b427b48e16b5e91360c919c9cf60cb").unwrap();
        assert_eq!(
            "SrLftu6GA4z6iV2RwiKCaf2xEHPW2ZCHT2ZbNGUeQXzVxQYpLtf",
            result
        );
    }

    #[test]
    fn test_to() -> Result<()> {
        let (key, _) = to_public_key("SrLftu6GA4z6iV2RwiKCaf2xEHPW2ZCHT2ZbNGUeQXzVxQYpLtf")?;
        assert_eq!(
            "03511c83916ac338835b07f6b9f7c0aa10b7b427b48e16b5e91360c919c9cf60cb",
            key
        );

        assert!(!is_wallet(
            "SrLftu6GA4z6iV2RwiKCaf2xEHPW2ZCHT2ZbNGUeQXzVxQYpLtF"
        ));

        Ok(())
    }
}

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
use super::util::{hex_str_to_bytes, ripemd160_raw, sha256_raw};
use super::Result;
use bs58;

// ----------------------------------------------------------------------------

pub fn new(input: &[u8]) -> String {
    let raw = ripemd160_raw(&sha256_raw(input));
    let mut tmp: Vec<u8> = Vec::with_capacity(1 + raw.len());
    tmp.push(0_u8);
    tmp.extend(raw);
    bs58::encode(&tmp).with_check().into_string()
}

pub fn from_hex(hexstr: &str) -> Result<String> {
    hex_str_to_bytes(hexstr).map(|x| new(&x))
}

pub fn is_wallet(wallet: &str) -> bool {
    bs58::decode(wallet).with_check(Some(0)).into_vec().is_ok()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from() {
        let result =
            from_hex("03511c83916ac338835b07f6b9f7c0aa10b7b427b48e16b5e91360c919c9cf60cb").unwrap();
        assert_eq!("1Lpgbz8o24ENRsZD3Rr5fVzJr2Ln4BBi5F", result);
    }

    #[test]
    fn test_to() -> Result<()> {
        assert!(is_wallet("1Lpgbz8o24ENRsZD3Rr5fVzJr2Ln4BBi5F"));

        assert!(!is_wallet("1Lpgbz8o24ENRsZD3Rr5fVzJr2Ln4BBi5f"));

        Ok(())
    }
}

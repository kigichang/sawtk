extern crate bs58;
use super::Result;
use super::util::{sha256_raw, hex_str_to_bytes};

// ----------------------------------------------------------------------------

fn checksum(input: &[u8]) -> Vec<u8> {
    let h = sha256_raw(input);
    let h2 = sha256_raw(&h);

    Vec::from(&h2[..4])
}

fn check_encode(input: &[u8], version: u8) -> String {
    let mut b = Vec::new();

    b.push(version);
    b.extend_from_slice(input);

    let cksum = checksum(&b);

    b.extend_from_slice(&cksum);
    bs58::encode(&b).into_string()
}

//fn check_decode(input: &str)

pub fn new(input: &[u8]) -> String {
    return check_encode(input, input[0]);
}

pub fn from_hex(hexstr: &str) -> Result<String> {
    hex_str_to_bytes(hexstr).map(|x| new(&x))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from() {
        let result = from_hex("03511c83916ac338835b07f6b9f7c0aa10b7b427b48e16b5e91360c919c9cf60cb").unwrap();
        assert_eq!("SrLftu6GA4z6iV2RwiKCaf2xEHPW2ZCHT2ZbNGUeQXzVxQYpLtf", result);
    }
}
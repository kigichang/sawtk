use sawtooth_sdk::signing::Signer;
use std::fmt;

pub struct Batcher<'a> {
    pub signer: &'a Signer<'a>,
}

impl<'a> Batcher<'a> {
    pub fn new(signer: &'a Signer) -> Self {
        Batcher { signer: signer }
    }
}

impl<'a> fmt::Display for Batcher<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.signer.get_public_key() {
            Ok(key) => write!(f, "batcher: {}", key.as_hex()),
            Err(e) => write!(f, "get batcher failure: {}", e),
        }
    }
}
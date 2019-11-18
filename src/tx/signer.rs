/*
use sawtooth_sdk::signing;
use sawtooth_sdk::messages::transaction::TransactionHeader;
use std::fmt;
use crate::{Result, Error};


pub struct Signer<'a> {
    signer: &'a signing::Signer<'a>,
}

impl<'a> fmt::Display for Signer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.get_public_key() {
            Ok(key) => write!(f, "signer: {}", key.as_hex()),
            Err(e) => write!(f, "get signer failure: {}", e),
        }
    }
}

impl<'a> Signer<'a> {
    
    pub fn new(signer: &'a signing::Signer<'a>) -> Self {
        Signer { signer: signer }
    }

    
    pub fn sign(&self, message: &[u8]) -> Result<String> {
        self.signer.sign(message).map_err(|e| Error::Signing(e))
    }

    pub fn get_public_key(&self) -> Result<String> {
        self.signer.get_public_key()
            .map(|key| key.as_hex())
            .map_err(|e| Error::Signing(e))
    }

    pub fn tx_header(&self, batcher_public_key: String, data: &super::Payload, dependencies: Vec<String>) -> Result<TransactionHeader> {
        let signer_public_key = self.get_public_key()?;
        Ok(data.tx_header(batcher_public_key, signer_public_key, dependencies))
    }
}
*/
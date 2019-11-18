use protobuf::{RepeatedField, Message};
use super::{Result, Error};
use sawtooth_sdk::messages::transaction::TransactionHeader;
use crate::util::{nonce, sha512_bytes};

pub mod batcher;
pub mod signer;

// ----------------------------------------------------------------------------

pub struct Payload {
    family_name: String,
    family_version: String,
    payload: Vec<u8>,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl Payload {
    pub fn new(family_name: String, family_version: String, msg: &dyn Message, inputs: Vec<String>, outputs: Vec<String>) -> Result<Self> {
        msg.write_to_bytes()
            .map(|b| {
                Payload { 
                    family_name: family_name,
                    family_version: family_version,
                    payload: b,
                    inputs: inputs,
                    outputs: outputs,
                }
            })
            .map_err(|e| Error::Protobuf(e))
    }

    pub fn tx_header(&self, batcher_public_key: String, signer_public_key: String, dependencies: Vec<String>) -> TransactionHeader {
        TransactionHeader {
            batcher_public_key: batcher_public_key,
            dependencies: RepeatedField::from_vec(dependencies),
            family_name: self.family_name.clone(),
            family_version: self.family_version.clone(),
            inputs: RepeatedField::from_slice(&self.inputs),
            nonce: nonce(),
            outputs: RepeatedField::from_slice(&self.outputs),
            payload_sha512: sha512_bytes(&self.payload),
            signer_public_key: signer_public_key,
            ..TransactionHeader::default()
        }
    }
}
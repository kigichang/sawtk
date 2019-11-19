use protobuf::{self,RepeatedField, Message};
use super::{Result, Error};
use sawtooth_sdk::messages::transaction::{TransactionHeader, Transaction};
use sawtooth_sdk::messages::batch::{BatchHeader, Batch, BatchList};
use crate::util::{nonce, sha512_bytes};
use super::signing::Signer;


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

    pub fn tx_header(&self, batcher_public_key: &str, signer_public_key: &str, dependencies: Vec<String>) -> TransactionHeader {
        TransactionHeader {
            batcher_public_key: String::from(batcher_public_key),
            dependencies: RepeatedField::from_vec(dependencies),
            family_name: self.family_name.clone(),
            family_version: self.family_version.clone(),
            inputs: RepeatedField::from_slice(&self.inputs),
            nonce: nonce(),
            outputs: RepeatedField::from_slice(&self.outputs),
            payload_sha512: sha512_bytes(&self.payload),
            signer_public_key: String::from(signer_public_key),
            ..TransactionHeader::default()
        }
    }
}

// ----------------------------------------------------------------------------

pub struct Builder<'a> {
    signer: &'a Signer,
}

impl<'a> Builder<'a> {
    pub fn new(signer: &'a Signer) -> Self {
        Builder { signer: signer }
    }

    pub fn header(&self, batcher_public_key: &str, data: &Payload, dependencies: Vec<String>) -> Result<TransactionHeader> {
        let signer_public_key = self.signer.get_public_key()?;

        Ok(data.tx_header(batcher_public_key, &signer_public_key, dependencies))
    }

    pub fn build(&self, batcher_public_key: &str, data: &Payload, dependencies: Vec<String>) -> Result<Transaction> {
        let header = self.header(batcher_public_key, data, dependencies)?;
        let mut header_bytes: Vec<u8> = Vec::new();

        match header.write_to_vec(&mut header_bytes) {
            Err(e) => Err(Error::Protobuf(e)),
            Ok(_) => {
                let sign = self.signer.sign(&header_bytes)?;
                Ok(Transaction {
                    header: header_bytes,
                    header_signature: sign,
                    payload: Vec::from(&(*data.payload)),
                    ..Transaction::default()
                })
            }
        }
    }
}

// ----------------------------------------------------------------------------

pub struct Batcher<'a> {
    signer: &'a Signer,
}

impl<'a> Batcher<'a> {
    pub fn new(signer: &'a Signer) -> Self {
        Batcher { signer: signer }
    }

    pub fn get_public_key(&self) -> Result<String> {
        self.signer.get_public_key()
    }

    fn header(&self, transactions: &Vec<Transaction>) -> Result<BatchHeader> {
        let pub_key = self.get_public_key()?;
        let ids:Vec<String> = transactions.iter().map(|x| String::from(&x.header_signature)).collect();


        Ok(
            BatchHeader {
                signer_public_key: pub_key,
                transaction_ids: RepeatedField::from(ids),
                ..BatchHeader::default()
            }
        )
    }

    pub fn build(&self, transactions: Vec<Transaction>) -> Result<Batch> {
        let header = self.header(&transactions)?;
        let mut header_bytes: Vec<u8> = Vec::new();

        match header.write_to_vec(&mut header_bytes) {
            Err(e) => Err(Error::Protobuf(e)),
            Ok(_) => {
                let sign = self.signer.sign(&header_bytes)?;
                Ok(
                    Batch {
                        header: header_bytes,
                        header_signature: sign,
                        transactions: RepeatedField::from(transactions),
                        ..Batch::default()
                    }
                )
            }
        }
    }

    pub fn to_list(batches: Vec<Batch>) -> BatchList {
        BatchList {
            batches: RepeatedField::from(batches),
            ..BatchList::default()
        }
    }
}

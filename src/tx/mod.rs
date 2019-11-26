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

use protobuf::{self,RepeatedField, Message};
use sawtooth_sdk::messages::transaction::{TransactionHeader, Transaction};
use sawtooth_sdk::messages::batch::{BatchHeader, Batch, BatchList};
use super::{Result, Error};
use super::signing::Signer;
use crate::util::{nonce, sha512_bytes};



// ----------------------------------------------------------------------------

pub struct Payload {
    family_name: String,
    family_version: String,
    payload: Vec<u8>,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl Payload {
    pub fn new(
        family_name:    String, 
        family_version: String, 
        msg:            &dyn Message, 
        inputs:         &[String], 
        outputs:        &[String]
    ) -> Result<Self> {

        msg.write_to_bytes()
            .map(|b| {
                Payload { 
                    family_name: family_name,
                    family_version: family_version,
                    payload: b,
                    inputs: Vec::from(inputs),
                    outputs: Vec::from(outputs),
                }
            })
            .map_err(|e| Error::Protobuf(e))
    }

    pub fn tx_header(
            &self, 
            batcher_public_key: &str, 
            signer_public_key:  &str, 
            dependencies:       &[String]
    ) -> TransactionHeader {

        TransactionHeader {
            batcher_public_key: String::from(batcher_public_key),
            dependencies: RepeatedField::from_slice(dependencies),
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

    pub fn header(
        &self,
        batcher_public_key: &str,
        data: &Payload,
        dependencies: &[String]
    ) -> Result<TransactionHeader> {

        let signer_public_key = self.signer.get_public_key()?;

        Ok(data.tx_header(batcher_public_key, &signer_public_key, dependencies))
    }

    pub fn build(
        &self, 
        batcher_public_key: &str, 
        data:               &Payload, 
        dependencies:       &[String]
    ) -> Result<Transaction> {

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

    fn header(&self, transactions: &[Transaction]) -> Result<BatchHeader> {
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

    pub fn build(&self, transactions: &[Transaction]) -> Result<Batch> {
        let header = self.header(transactions)?;
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

    pub fn to_list(batches: &[Batch]) -> BatchList {
        BatchList {
            batches: RepeatedField::from_slice(batches),
            ..BatchList::default()
        }
    }
}

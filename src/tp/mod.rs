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

use crate::messages::request::TPRequest;
use protobuf::{self, Message};
use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::handler::TransactionContext;
use std::collections::HashMap;

// -----------------------------------------------------------------------------

#[macro_export]
macro_rules! invalid_transaction {
    ($($x:tt)*) => {
        ApplyError::InvalidTransaction(
            format!($($x)*)
        )
    }
}
// -----------------------------------------------------------------------------

pub struct States {
    data: HashMap<String, Vec<u8>>,
}

impl States {
    pub fn contains(&self, address: &str) -> bool {
        self.data.contains_key(address)
    }

    pub fn get<T: protobuf::Message>(&self, address: &str) -> Result<T, ApplyError> {
        let mut msg = T::new();
        self.get_message(address, &mut msg)?;
        Ok(msg)
    }

    pub fn get_message(
        &self,
        address: &str,
        msg: &mut dyn protobuf::Message,
    ) -> Result<(), ApplyError> {
        let bytes = self
            .data
            .get(address)
            .ok_or(invalid_transaction!("{} not found", address))?;
        msg.merge_from_bytes(bytes)
            .map_err(|e| invalid_transaction!("{}", e))?;
        Ok(())
    }
}

impl From<Vec<(String, Vec<u8>)>> for States {
    fn from(result: Vec<(String, Vec<u8>)>) -> Self {
        States {
            data: result
                .into_iter()
                .map(|rs| (rs.0, rs.1))
                .collect::<HashMap<_, _>>(),
        }
    }
}

// -----------------------------------------------------------------------------

pub fn get_state_entries(
    ctx: &dyn TransactionContext,
    addresses: Vec<String>,
) -> Result<States, ApplyError> {
    let result = ctx
        .get_state_entries(&addresses)
        .map_err(|e| invalid_transaction!("{}", e))?;
    Ok(States::from(result))
}

pub fn get_state_entry<T: protobuf::Message>(
    ctx: &dyn TransactionContext,
    address: &str,
) -> Result<T, ApplyError> {
    get_state_entries(ctx, vec![address.to_string()])?.get::<T>(address)
}

pub fn delete_state_entries(
    ctx: &dyn TransactionContext,
    addresses: &[String],
) -> Result<Vec<String>, ApplyError> {
    ctx.delete_state_entries(addresses)
        .map_err(|e| invalid_transaction!("{}", e))
}

pub fn delete_state_entry(
    ctx: &dyn TransactionContext,
    address: &str,
) -> Result<Option<String>, ApplyError> {
    ctx.delete_state_entry(address)
        .map_err(|e| invalid_transaction!("{}", e))
}

pub fn set_state_entries(
    ctx: &dyn TransactionContext,
    data: Vec<(String, &dyn Message)>,
) -> Result<(), ApplyError> {
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();

    for rs in data {
        let bytes =
            rs.1.write_to_bytes()
                .map_err(|e| invalid_transaction!("{}", e))?;
        entries.push((rs.0, bytes));
    }

    ctx.set_state_entries(entries)
        .map_err(|e| invalid_transaction!("{}", e))
}

pub fn set_state_entry(
    ctx: &dyn TransactionContext,
    address: String,
    data: &dyn Message,
) -> Result<(), ApplyError> {
    let bytes = data
        .write_to_bytes()
        .map_err(|e| invalid_transaction!("{}", e))?;
    ctx.set_state_entry(address, bytes)
        .map_err(|e| invalid_transaction!("{}", e))
}

pub fn add_event(
    ctx: &dyn TransactionContext,
    event_type: String,
    attributes: Vec<(String, String)>,
    data: &dyn Message,
) -> Result<(), ApplyError> {
    let bytes = data
        .write_to_bytes()
        .map_err(|e| invalid_transaction!("{}", e))?;
    ctx.add_event(event_type, attributes, &bytes)
        .map_err(|e| invalid_transaction!("{}", e))
}

// -----------------------------------------------------------------------------

pub fn to_tp_request(req: &TpProcessRequest) -> Result<TPRequest, ApplyError> {
    to_message::<TPRequest>(&req.payload)
}

pub fn to_message<T: protobuf::Message>(bytes: &[u8]) -> Result<T, ApplyError> {
    protobuf::parse_from_bytes::<T>(bytes).map_err(|e| invalid_transaction!("{}", e))
}

// -----------------------------------------------------------------------------
pub trait Validate: protobuf::Message {
    fn validate(&mut self) -> Result<(), ApplyError>;
}

impl Validate for TPRequest {
    fn validate(&mut self) -> Result<(), ApplyError> {
        Ok(())
    }
}

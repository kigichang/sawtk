use std::collections::HashMap;

use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::handler::TransactionContext;
use protobuf::{self, Message};
use crate::messages::request::TPRequest;

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

    pub fn to<T: protobuf::Message>(&self, address: &str) -> Result<T, ApplyError> {
        let mut msg = T::new();
        self.to_message(address, &mut msg)?;
        Ok(msg)
    }

    pub fn to_message(&self, address: &str, msg: &mut dyn protobuf::Message) -> Result<(), ApplyError> {
        let bytes = self.data.get(address).ok_or(
            invalid_transaction!("{} not found", address)
        )?;
        msg.merge_from_bytes(bytes).map_err(|e| 
            invalid_transaction!("{}", e)
        )?;
        Ok(())
    }
}

impl From<Vec<(String, Vec<u8>)>> for States {
    fn from(result: Vec<(String, Vec<u8>)>) -> Self {
        let mut data = HashMap::new();

        for rs in result.into_iter() {
            data.insert(rs.0, rs.1);
        }

        States { data: data }
    }
}


// -----------------------------------------------------------------------------

pub fn get_state_entries(ctx: &dyn TransactionContext, addresses: Vec<String>) -> Result<(States), ApplyError> {
    let result = ctx.get_state_entries(&addresses).map_err(|e| invalid_transaction!("{}", e))?;
    Ok(States::from(result))
}

pub fn get_state_entry<T: protobuf::Message>(ctx: &dyn TransactionContext, address: &str) -> Result<T, ApplyError> {
    get_state_entries(ctx, vec![address.to_string()])?
        .to::<T>(address)
}


pub fn delete_state_entries(ctx: &dyn TransactionContext, addresses: &[String]) -> Result<Vec<String>, ApplyError> {
    ctx.delete_state_entries(addresses).map_err(|e| invalid_transaction!("{}", e))
}

pub fn delete_state_entry(ctx: &dyn TransactionContext, address: &str) -> Result<Option<String>, ApplyError> {
    ctx.delete_state_entry(address).map_err(|e| invalid_transaction!("{}", e))
}


pub fn set_state_entries(ctx: &dyn TransactionContext, data: Vec<(String, &dyn Message)>) -> Result<(), ApplyError> {
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();

    for rs in data {
        let bytes = rs.1.write_to_bytes().map_err(|e| invalid_transaction!("{}", e))?;
        entries.push((rs.0, bytes));
    }

    ctx.set_state_entries(entries).map_err(|e| invalid_transaction!("{}", e))
}

pub fn set_state_entry(ctx: &dyn TransactionContext, address: String, data: &dyn Message) -> Result<(), ApplyError> {
    let bytes = data.write_to_bytes().map_err(|e| invalid_transaction!("{}", e))?;
    ctx.set_state_entry(address, bytes).map_err(|e| invalid_transaction!("{}", e))
}

pub fn add_event(ctx: &dyn TransactionContext, event_type: String, attributes: Vec<(String, String)>, data: &dyn Message) -> Result<(), ApplyError> {
    let bytes = data.write_to_bytes().map_err(|e| invalid_transaction!("{}", e))?;
    ctx.add_event(event_type, attributes, &bytes).map_err(|e| invalid_transaction!("{}", e))
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
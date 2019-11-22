use std::collections::HashMap;
use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::handler::TransactionContext;
use protobuf::{self, Message};
//use crate::ns::Namespace;
use crate::messages::request::TPRequest;

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
        let bytes = self.data.get(address).ok_or(ApplyError::InvalidTransaction(format!("{} not found", address)))?;
        msg.merge_from_bytes(bytes).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;
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
    let result = ctx.get_state_entries(&addresses).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;
    Ok(States::from(result))
}

pub fn get_state_entry<T: protobuf::Message>(ctx: &dyn TransactionContext, address: &str) -> Result<T, ApplyError> {
    get_state_entries(ctx, vec![address.to_string()])?
        .to::<T>(address)
}
/*
pub fn get_state_entry(ctx: &dyn TransactionContext, address: &str, data: Box<&mut dyn Message>) -> Result<(), ApplyError> {

    let result = ctx.get_state_entry(address).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;

    match result {
        None => Err(ApplyError::InvalidTransaction(format!("{} not found", address))),

        Some(bytes) => {
            data.merge_from_bytes(&bytes).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))
        },
    }
}
*/

pub fn delete_state_entries(ctx: &dyn TransactionContext, addresses: &[String]) -> Result<Vec<String>, ApplyError> {
    ctx.delete_state_entries(addresses).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))
}

pub fn delete_state_entry(ctx: &dyn TransactionContext, address: &str) -> Result<Option<String>, ApplyError> {
    ctx.delete_state_entry(address).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))
}


pub fn set_state_entries(ctx: &dyn TransactionContext, data: Vec<(String, &dyn Message)>) -> Result<(), ApplyError> {
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();

    for rs in data {
        let mut bytes: Vec<u8> = Vec::new();
        rs.1.write_to_vec(&mut bytes).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;
        entries.push((rs.0, bytes));
    }

    ctx.set_state_entries(entries).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))
}

pub fn set_state_entry(ctx: &dyn TransactionContext, address: String, data: &dyn Message) -> Result<(), ApplyError> {
    let bytes = data.write_to_bytes().map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;
    ctx.set_state_entry(address, bytes).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))
}

pub fn add_event(ctx: &dyn TransactionContext, event_type: String, attributes: Vec<(String, String)>, data: &dyn Message) -> Result<(), ApplyError> {
    let bytes = data.write_to_bytes().map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;
    ctx.add_event(event_type, attributes, &bytes).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))
}

// -----------------------------------------------------------------------------

pub fn to_tp_request(req: &TpProcessRequest) -> Result<TPRequest, ApplyError> {
    protobuf::parse_from_bytes::<TPRequest>(&req.payload).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))
}

// -----------------------------------------------------------------------------
/*
pub struct Context<'a> {
    ctx: &'a dyn TransactionContext,
    cmd: i32,
    signer: String,
}

impl <'a> Context <'a> {

    pub fn command(&self) -> i32 {
        self.cmd
    }

    pub fn signer_public_key(&self) -> String {
        self.signer.clone()
    }

    pub fn get_state_entries(&self, data: &mut HashMap<String, Box<dyn Message>>) -> Result<(), ApplyError> {
        get_state_entries(self.ctx, data)
    }

    pub fn get_state_entry(&self, address: &str, data: Box<&mut dyn Message>) -> Result<(), ApplyError> {
        get_state_entry(self.ctx, address, data)
    }

    pub fn delete_state_entry(&self, address: &str) -> Result<Option<String>, ApplyError> {
        delete_state_entry(self.ctx, address)
    }

    pub fn delete_state_entries(&self, addresses: &[String]) -> Result<Vec<String>, ApplyError> {
        delete_state_entries(self.ctx, addresses)
    }

    pub fn set_state_entries(&self, data: Vec<(String, &dyn Message)>) -> Result<(), ApplyError> {
        set_state_entries(self.ctx, data)
    }

    pub fn set_state_entry(&self, address: String, data: &dyn Message) -> Result<(), ApplyError> {
        set_state_entry(self.ctx, address, data)
    }

    pub fn add_event(&self, event_type: String, attributes: Vec<(String, String)>, data: &dyn Message) -> Result<(), ApplyError> {
        add_event(self.ctx, event_type, attributes, data)
    }
}

// -----------------------------------------------------------------------------

type HandleFunc = Box<(dyn (Fn(&mut Context, &TPRequest) -> Result<(), ApplyError>) + 'static)>;

pub fn make_handle_func(f: impl Fn(&mut Context, &TPRequest) -> Result<(), ApplyError> + 'static) -> HandleFunc {
    Box::new(f) as HandleFunc
}

pub struct Handler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<dyn Namespace>,
    handles: HashMap<i32, HandleFunc>,
}

impl Handler {

    pub fn new(family_name: &str, family_versions: &[String], namespaces: Vec<dyn Namespace>) -> Self {
        Handler { 
            family_name: String::from(family_name),
            family_versions: Vec::from(family_versions),
            namespaces: namespaces,
            handles: HashMap::new(),
        }
    }


    pub fn add(&mut self, command: i32,  f: impl Fn(&mut Context, &TPRequest) -> Result<(), ApplyError> + 'static) {
        self.handles.insert(command, make_handle_func(f));
    }
}

impl<'a> TransactionHandler for Handler<'a> {
    fn family_name(&self) -> String {
        self.family_name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family_versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
        self.namespaces.map(|ns| ns.prefix()).collect::<Vec<_>>()
    }

    fn apply(&self, request: &TpProcessRequest, context: &mut dyn TransactionContext) -> Result<(), ApplyError> {
        let req = protobuf::parse_from_bytes::<TPRequest>(&request.payload)
            .map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;

        match self.handles.get(&req.cmd) {
            None => {
                Err(
                    ApplyError::InvalidTransaction(
                        format!("unknown cmd: {}", req.cmd)
                    )
                )
            },
            Some(f) => f(&mut Context { cmd: req.cmd, ctx: context, signer: request.get_header().signer_public_key.clone()}, &req),
        }
    }
}
*/
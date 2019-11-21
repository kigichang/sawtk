use std::collections::{HashMap, HashSet};
use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::handler::{TransactionHandler, TransactionContext};
use protobuf::{self, Message};
use crate::ns::Namespace;
use crate::messages::request::TPRequest;

// -----------------------------------------------------------------------------

pub fn get_state_entries(ctx: &dyn TransactionContext, data: &mut HashMap<String, Box<dyn Message>>) -> Result<(), ApplyError> {

    let addresses = data.keys().cloned().collect::<Vec<_>>();

    let result = ctx.get_state_entries(&addresses).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;

    let x1 = data.keys().cloned().collect::<HashSet<_>>();
    let x2 = result.iter().map(|x| String::from(&x.0)).collect::<HashSet<_>>();
    let not_found = x1.difference(&x2);

    for x in not_found {
        data.remove(x);
    }


    for rs in result {
        let msg = data.get_mut(&rs.0);
        if msg.is_none() {
            continue;
        }

        msg.unwrap()
            .merge_from_bytes(&rs.1)
            .map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;
    }

    Ok(())
}

pub fn get_state_entry(ctx: &dyn TransactionContext, address: &str, data: Box<&mut dyn Message>) -> Result<(), ApplyError> {

    let result = ctx.get_state_entry(address).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;

    match result {
        None => Err(ApplyError::InvalidTransaction(format!("{} not found", address))),

        Some(bytes) => {
            data.merge_from_bytes(&bytes).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))
        },
    }
}

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

pub struct Handler<'a> {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<&'a dyn Namespace>,
    handles: HashMap<i32, HandleFunc>,
}

impl<'a> Handler<'a> {

    pub fn new(family_name: &str, family_versions: &[String], namespaces: &[&'static dyn Namespace]) -> Self {
        Handler { 
            family_name: String::from(family_name),
            family_versions: Vec::from(family_versions),
            namespaces: Vec::from(namespaces),
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
        self.namespaces.iter().map(|ns| ns.prefix()).collect::<Vec<_>>()
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
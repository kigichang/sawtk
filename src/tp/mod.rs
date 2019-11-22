use std::collections::HashMap;
use std::fmt;

use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::handler::TransactionContext;
use sawtooth_sdk::processor::handler::TransactionHandler;
use protobuf::{self, Message};
use crate::messages::request::TPRequest;

use super::ns::{self, Namespace};

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


pub fn delete_state_entries(ctx: &dyn TransactionContext, addresses: &[String]) -> Result<Vec<String>, ApplyError> {
    ctx.delete_state_entries(addresses).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))
}

pub fn delete_state_entry(ctx: &dyn TransactionContext, address: &str) -> Result<Option<String>, ApplyError> {
    ctx.delete_state_entry(address).map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))
}


pub fn set_state_entries(ctx: &dyn TransactionContext, data: Vec<(String, &dyn Message)>) -> Result<(), ApplyError> {
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();

    for rs in data {
        let bytes = rs.1.write_to_bytes().map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;
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
pub trait Validate: protobuf::Message {
    fn validate(&self) -> Result<(), ApplyError>;
    fn new() -> Box<Self> where Self: Sized;
}

impl Validate for TPRequest {
    fn validate(&self) -> Result<(), ApplyError> {
        Ok(())
    }

    fn new() -> Box<Self> {
        Box::new(TPRequest::new())
    }
}

// -----------------------------------------------------------------------------
pub struct Context<'a> {
    ctx: &'a mut dyn TransactionContext,
    origin: &'a TpProcessRequest,
    cmd: i32,
}

impl<'a> Context<'a> {

    pub fn command(&mut self) -> i32 {
        self.cmd
    }

    pub fn get_origin_request(&mut self) -> &TpProcessRequest {
        self.origin
    }

    pub fn get_signer_public_key(&mut self) -> &str {
        self.origin.get_header().get_signer_public_key()
    }

    pub fn get_batcher_public_key(&mut self) -> &str {
        self.origin.get_header().get_batcher_public_key()
    }


    pub fn get_state_entries(&mut self, addresses: Vec<String>) -> Result<(States), ApplyError> {
        get_state_entries(self.ctx, addresses)
    }

    pub fn get_state_entry<T: protobuf::Message>(&mut self, address: &str) -> Result<T, ApplyError> {
        get_state_entry(self.ctx, address)
    }


    pub fn delete_state_entries(&mut self, addresses: &[String]) -> Result<Vec<String>, ApplyError> {
        delete_state_entries(self.ctx, addresses)
    }

    pub fn delete_state_entry(&mut self, address: &str) -> Result<Option<String>, ApplyError> {
        delete_state_entry(self.ctx, address)
    }


    pub fn set_state_entries(&mut self, data: Vec<(String, &dyn Message)>) -> Result<(), ApplyError> {
        set_state_entries(self.ctx, data)
    }

    pub fn set_state_entry(&mut self, address: String, data: &dyn Message) -> Result<(), ApplyError> {
        set_state_entry(self.ctx, address, data)
    }


    pub fn add_event(&mut self, event_type: String, attributes: Vec<(String, String)>, data: &dyn Message) -> Result<(), ApplyError> {
        add_event(self.ctx, event_type, attributes, data)
    }

}

// -----------------------------------------------------------------------------
type Constructor = Box<(dyn (Fn() -> Box<dyn Validate>) + 'static)>;

type HandleFunc = Box<(dyn (Fn(&mut Context, &dyn Validate) -> Result<(), ApplyError>) + 'static)>;


pub fn make_constructor(f: impl Fn() -> Box<dyn Validate> + 'static) -> Constructor {
    Box::new(f) as Constructor
}

pub fn make_handle_func(f: impl Fn(&mut Context, &dyn Validate) -> Result<(), ApplyError> + 'static) -> HandleFunc {
    Box::new(f) as HandleFunc
}

pub struct Handler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<Box<dyn Namespace>>,
    handles: HashMap<i32, (HandleFunc, Constructor)>,
}

impl Handler {
    pub fn new(family_name: &str, family_versions: &[String], namespaces: &[&str]) -> Self {
        let all_ns: Vec<Box<dyn Namespace>> = namespaces.iter().map(|x| ns::new(x)).collect::<Vec<_>>();

        Handler { 
            family_name: family_name.to_string(),
            family_versions: Vec::from(family_versions),
            namespaces: all_ns,
            handles: HashMap::new(),
        }
    }

    pub fn add(&mut self, cmd: i32, f: impl Fn(&mut Context, &dyn Validate) -> Result<(), ApplyError> + 'static, c: impl Fn() -> Box<dyn Validate> + 'static) {
        self.handles.insert(cmd, (make_handle_func(f), make_constructor(c)));
    }
}

impl fmt::Display for Handler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
            "family_name: {}, family_versions: {:?}, namespaces: {}", 
            self.family_name, 
            self.family_versions, 
            self.namespaces.iter()
                .map(|n| format!("{}", n))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl TransactionHandler for Handler {
    fn family_name(&self) -> String {
        self.family_name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family_versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
       self.namespaces.iter().map(|n| n.prefix().to_string()).collect::<Vec<_>>()
    }

    fn apply(&self, request: &TpProcessRequest, context: &mut dyn TransactionContext) -> Result<(), ApplyError> {
        let req = to_tp_request(request)?;
        let cmd = req.cmd;

        let (h, c) = self.handles.get(&cmd).ok_or(ApplyError::InvalidTransaction(format!("unknown command {}", cmd)))?;
        let new_req: Box<dyn Validate> = 
            if req.payload.len() == 0 {
                Box::new(req)
            } else {
                let mut tmp = c();
                tmp.merge_from_bytes(&req.payload)
                    .map_err(|e| ApplyError::InvalidTransaction(format!("{}", e)))?;
                tmp
            };

        new_req.validate()?;
        
        let mut ctx = Context {
            ctx: context,
            origin: request,
            cmd: cmd,
        };

        h(&mut ctx, &*new_req)
    }
}

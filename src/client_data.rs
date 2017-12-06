use std::net::Ipv4Addr;
use std::collections::HashMap;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ClientState {
    UNKOWN,
    ALIVE,
    REGISTERED,
    ALREADYR_EGISTERED,
    REGISTRATION_INFORMATION_INVALID,
    UNREGISTERED,
    ERROR
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionClient {
    pub gid: u8,
    pub ip: Ipv4Addr,
    pub name: String,
    pub port: u16,
    pub root: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistrationRequest {
    pub ip: Ipv4Addr,
    pub name: String,
    pub port: u16,
    pub root: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrepareMessage {
    pub node: Node,
    pub clientRequest: RegistrationRequest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub ip: Ipv4Addr,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrepareAcceptMessage {
    pub ip: Ipv4Addr,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResolvedMessage {
    pub node: Node,
    pub clientlist: Vec<TransactionClient>,
}

#[derive(Debug)]
pub struct ClientListHandler {
    list: HashMap<u32, TransactionClient>,
    accept_prepare: bool,
}



impl ClientListHandler {
    pub fn new() -> Self {
        ClientListHandler {
            list: HashMap::new(),
            accept_prepare: true,
        }
    }

    pub fn list(&self) -> Vec<TransactionClient> {
        self.list.values().cloned().collect()
    }
}
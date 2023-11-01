use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread::ThreadId,
};
pub mod agent;
pub mod entity;
pub mod service;
//pub mod organization;

use entity::messaging::Message;

type ID = ThreadId;
type ThreadPriority = i32;
type StackSize = usize;
type TX = Sender<Message>;
type RX = Receiver<Message>;
type Directory = HashMap<String, (String, TX)>; //can be expanded into different dir types for agents, AMS or DF if present

pub const DEFAULT_STACK: usize = 8;
pub const MAX_PRIORITY: ThreadPriority = 99;
pub const MAX_SUBSCRIBERS: usize = 64;

pub enum ErrorCode {
    NoError,
    Found,
    HandleNone,
    ListFull,
    Duplicated,
    NotFound,
    Timeout,
    Invalid,
    NotRegistered,
}

pub struct Platform {
    name: String,
    white_pages: Arc<Mutex<Directory>>,
}

impl Platform {
    fn new(name: String) -> Self {
        let white_pages: Arc<Mutex<Directory>> =
            Arc::new(Mutex::new(Directory::with_capacity(MAX_SUBSCRIBERS)));
        Self { name, white_pages }
    }
}

/*struct Parent(pub ThreadId);
impl Parent {

}*/

/*
fn hey()->agents::base::AID{
    let x = agents::base::new("yo".to_string(), 1, 20);
    x.get_aid().unwrap()
}
*/

/*Agents have generic data, for implementation the user must:
- create structs for each agent with each behavior data:
struct A {a,b,c}
struct B {d,e,f}
- encapsulate the struct in a user define enum:

enum AgentData{
    AgentA(A)
    AgentB(B)
}

AgentData enum must be passed as type when instantiating agents
*/

use std::{
    collections::HashMap,
    sync::{
        atomic::AtomicBool,
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    }, thread::JoinHandle,
};

pub mod agent;
pub mod entity;
pub mod service;
//pub mod organization;

use entity::{messaging::Message, Description};

type ThreadPriority = i32;
type StackSize = usize;
type TX = Sender<Message>;
type RX = Receiver<Message>;
type Directory = HashMap<String, Description>; //can be expanded into different dir types for agents, AMS or DF if present

pub(crate) struct ControlBlock {
    handle: Option<JoinHandle<()>>,
    init: AtomicBool,
    suspend: AtomicBool,
    quit: AtomicBool,
}

type ControlBlockDirectory = HashMap<String, ControlBlock>;
//set directory entry type: must include name, Thread ID, TX, Join Handle

pub const DEFAULT_STACK: usize = 8;
pub const MAX_PRIORITY: ThreadPriority = 99;
pub const MAX_SUBSCRIBERS: usize = 64;

#[derive(PartialEq)]
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
    control_block_directory: Arc<Mutex<ControlBlockDirectory>>,
}

impl Platform {
    fn new(name: String) -> Self {
        let white_pages: Arc<Mutex<Directory>> =
            Arc::new(Mutex::new(Directory::with_capacity(MAX_SUBSCRIBERS)));
        let control_block_directory: Arc<Mutex<ControlBlockDirectory>> = Arc::new(Mutex::new(
            ControlBlockDirectory::with_capacity(MAX_SUBSCRIBERS),
        ));
        Self {
            name,
            white_pages,
            control_block_directory,
        }
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

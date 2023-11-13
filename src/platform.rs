use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, SyncSender},
        Arc, Mutex, RwLock,
    },
    thread::JoinHandle,
};

pub mod agent;
pub mod entity;
pub mod service;
//pub mod organization;

use {
    agent::ControlBlock,
    entity::{messaging::Message, Description},
};

type ThreadPriority = i32;
type StackSize = usize;
type TX = SyncSender<Message>;
type RX = Receiver<Message>;
type Directory = HashMap<String, Description>; //can be expanded into different dir types for agents, AMS or DF if present
type ControlBlockDirectory = HashMap<String, ControlBlock>;
type HandleDirectory = HashMap<String, JoinHandle<()>>;
type StateDirectory = HashMap<String, AgentState>;
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

#[derive(PartialEq, Clone, Copy)]
pub enum AgentState {
    Waiting,
    Active,
    Suspended,
    Initiated,
}

pub struct Platform {
    name: String,
    white_pages: Arc<RwLock<Directory>>,
    control_block_directory: Arc<RwLock<ControlBlockDirectory>>,
    handle_directory: Arc<Mutex<HandleDirectory>>,
    state_directory: Arc<RwLock<StateDirectory>>,
}

impl Platform {
    fn new(name: String) -> Self {
        let white_pages: Arc<RwLock<Directory>> =
            Arc::new(RwLock::new(Directory::with_capacity(MAX_SUBSCRIBERS)));
        let control_block_directory: Arc<RwLock<ControlBlockDirectory>> = Arc::new(RwLock::new(
            ControlBlockDirectory::with_capacity(MAX_SUBSCRIBERS),
        ));
        let handle_directory: Arc<Mutex<HandleDirectory>> =
            Arc::new(Mutex::new(HandleDirectory::with_capacity(MAX_SUBSCRIBERS)));
        let state_directory: Arc<RwLock<StateDirectory>> =
            Arc::new(RwLock::new(StateDirectory::with_capacity(MAX_SUBSCRIBERS)));
        Self {
            name,
            white_pages,
            control_block_directory,
            handle_directory,
            state_directory,
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

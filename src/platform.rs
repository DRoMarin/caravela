use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, SyncSender},
        Arc, Mutex, RwLock,
    },
    thread::JoinHandle,
};

use self::service::{DefaultConditions, Service};

pub mod agent;
pub mod entity;
pub mod service;
//pub mod organization;

use thread_priority::*;
use {
    agent::ControlBlock,
    entity::{messaging::Message, Description},
};

//type ThreadPriority = i32;
type Priority = ThreadPriorityValue;
type StackSize = usize;
type TX = SyncSender<Message>;
type RX = Receiver<Message>;
type Directory = HashMap<String, Description>; //can be expanded into different dir types for agents, AMS or DF if present
type ControlBlockDirectory = HashMap<String, ControlBlock>;
type HandleDirectory = HashMap<String, JoinHandle<()>>;
type StateDirectory = HashMap<String, AgentState>;
//set directory entry type: must include name, Thread ID, TX, Join Handle

pub const DEFAULT_STACK: usize = 8;
pub const MAX_PRIORITY: u8 = 99;
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
    pub fn new(name: String) -> Self {
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
    pub fn boot(&self) -> Result<(), &str> {
        let default = service::DefaultConditions;
        let mut ams = service::ams::AMS::<DefaultConditions>::new(&self, default);
        let ams_name = ams.service_hub.aid.get_name();
        let ams_handle = spawn(
            ThreadPriority::Crossplatform(ams.service_hub.resources.get_priority()),
            move |_| {
                println!("\nBOOTING AMS: {}\n", ams.service_hub.aid.get_name());
                ams.service_function();
            },
        );

        if ams_handle.is_finished() {
            return Err("AMS ended");
        }

        self.handle_directory
            .lock()
            .unwrap()
            .insert(ams_name, ams_handle);
        Ok(())
    }
}

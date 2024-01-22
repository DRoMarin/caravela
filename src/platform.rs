use self::{
    agent::{
        behavior::{execute, private::TaskControl, Behavior},
        Agent,
    },
    entity::Entity,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::AtomicBool,
        mpsc::{Receiver, SyncSender},
        Arc, Barrier, RwLock,
    },
    thread::JoinHandle,
};
use thread_priority::*;
use {
    agent::ControlBlock,
    entity::{messaging::Message, Description},
    service::{DefaultConditions, Service},
};

pub mod agent;
pub mod entity;
pub mod service;
//pub mod organization;

type Priority = ThreadPriorityValue;
type StackSize = usize;
type TX = SyncSender<Message>;
type RX = Receiver<Message>;
type Directory = HashMap<String, Description>; //can be expanded into different dir types for agents, AMS or DF if present
type ControlBlockDirectory = HashMap<String, Arc<ControlBlock>>;
type HandleDirectory = HashMap<String, JoinHandle<()>>;
type StateDirectory = HashMap<String, AgentState>;
//set directory entry type: must include name, Thread ID, TX, Join Handle

pub const DEFAULT_STACK: usize = 8;
pub const MAX_PRIORITY: u8 = 99;
pub const MAX_SUBSCRIBERS: usize = 64;

#[derive(PartialEq, Debug)]
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
    //pub refself: Arc<RwLock<Self>>,
    pub(crate) ams_aid: Option<Description>,
    pub(crate) shared_record: Arc<RwLock<SharedRecord>>,
    pub(crate) private_record: Arc<RwLock<PrivateRecord>>,
}
pub(crate) struct PrivateRecord {
    pub(crate) handle_directory: HandleDirectory, //accessed only by AMS
    pub(crate) white_pages_directory: Directory,  //accessed only by AMS
    pub(crate) control_block_directory: ControlBlockDirectory, //accessed only by AMS
}
pub(crate) struct SharedRecord {
    pub(crate) name: String,
    pub(crate) state_directory: StateDirectory, //modified only by Agents, accessed by All
}

impl Platform {
    pub fn new(name: String) -> Self {
        let handle_directory: HandleDirectory = HandleDirectory::with_capacity(MAX_SUBSCRIBERS);
        let control_block_directory: ControlBlockDirectory =
            ControlBlockDirectory::with_capacity(MAX_SUBSCRIBERS);
        let white_pages_directory: Directory = Directory::with_capacity(MAX_SUBSCRIBERS);
        let state_directory: StateDirectory = StateDirectory::with_capacity(MAX_SUBSCRIBERS);
        let shared_record = Arc::new(RwLock::new(SharedRecord {
            name,
            state_directory,
        }));
        let private_record = Arc::new(RwLock::new(PrivateRecord {
            handle_directory,
            white_pages_directory,
            control_block_directory,
        }));
        Self {
            ams_aid: None,
            shared_record,
            private_record,
        }
    }
    pub fn boot(&mut self) -> Result<(), &str> {
        let default = service::DefaultConditions;
        let mut ams = service::ams::AMS::<DefaultConditions>::new(&self, default);
        let ams_name = "AMS".to_string();
        let mut platform = self.private_record.write().unwrap();
        platform
            .white_pages_directory
            .insert(ams_name.clone(), ams.get_aid());
        self.ams_aid = Some(ams.get_aid());
        let ams_handle = std::thread::Builder::new().spawn_with_priority(
            ThreadPriority::Crossplatform(ams.service_hub.resources.get_priority()),
            move |_| {
                println!("\nBOOTING AMS: {}\n", ams.service_hub.aid.get_name());
                ams.service_function();
            },
        );
        /*if ams_handle.is_finished() {
            return Err("AMS ended");
        }*/
        if let Ok(handle) = ams_handle {
            platform.handle_directory.insert(ams_name, handle);
        }
        Ok(())
    }
    pub fn add<T>(
        &mut self,
        nickname: String,
        priority: u8,
        stack_size: usize,
        data: T,
    ) -> Result<Agent<T>, &str> {
        let tcb = Arc::new(ControlBlock {
            //init: AtomicBool::new(false),
            init: Barrier::new(2),
            suspend: AtomicBool::new(false),
            quit: AtomicBool::new(false),
        });
        let _hap = self.shared_record.read().unwrap().name.clone();
        let platform = self.shared_record.clone();
        let agent_creation = Agent::new(
            nickname.clone(),
            priority,
            stack_size,
            data,
            platform,
            tcb.clone(),
        );
        match agent_creation {
            Ok(mut agent) => {
                self.private_record
                    .write()
                    .unwrap()
                    .control_block_directory
                    .insert(nickname.clone(), tcb);
                agent
                    .directory
                    .insert("AMS".to_string(), self.ams_aid.clone().unwrap());
                Ok(agent)
            }
            Err(x) => Err(x),
        }
    }
    pub fn start(
        &mut self,
        agent: impl Behavior + TaskControl + Send + 'static,
    ) -> Result<(), &str> {
        let nickname = agent.get_nickname();
        let prio = agent.get_resources().get_priority();
        let mut platform = self.private_record.write().unwrap();
        let agent_handle = std::thread::Builder::new()
            .spawn_with_priority(ThreadPriority::Crossplatform(prio), move |_| execute(agent));
        if let Ok(handle) = agent_handle {
            platform.handle_directory.insert(nickname, handle);
        }
        Ok(())
    }
    //COULD ADD PLATFORM FUNCTIONS AND CALL THEM FROM AMS AGENT
}

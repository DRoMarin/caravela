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
        Arc, Mutex, RwLock,
    },
    thread::JoinHandle,
};
use thread_priority::*;
//use self::agent::behavior::{Behavior, Exec};
use {
    agent::ControlBlock,
    entity::{messaging::Message, Description},
    service::{DefaultConditions, Service},
};

pub mod agent;
pub mod entity;
pub mod service;
//pub mod organization;

//type ThreadPriority = i32;
type Priority = ThreadPriorityValue;
type StackSize = usize;
type TX = SyncSender<Message>;
type RX = Receiver<Message>;
type Directory = HashMap<String, Description>; //can be expanded into different dir types for agents, AMS or DF if present
                                               //type ControlBlockDirectory = HashMap<String, Arc<ControlBlock>>;
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
    //pub refself: Arc<RwLock<Self>>,
    pub(crate) mr: Arc<RwLock<MasterRecord>>,
}
pub(crate) struct MasterRecord {
    pub(crate) name: String,
    pub(crate) handle_directory: HandleDirectory, //modified by start
    pub(crate) control_block_directory: ControlBlockDirectory, //modified by add
    pub(crate) white_pages_directory: Directory,  //modified by AMS
    pub(crate) state_directory: StateDirectory,   //modified by Agents

                                                  /*handle_directory: Arc<Mutex<HandleDirectory>>, //modified by start
                                                  control_block_directory: Arc<RwLock<ControlBlockDirectory>>, //modified by add
                                                  white_pages_directory: Arc<RwLock<Directory>>, //modified by AMS
                                                  state_directory: Arc<RwLock<StateDirectory>>,  //modified by Agents*/
}

impl Platform {
    pub fn new(name: String) -> Self {
        /*let handle_directory: Arc<Mutex<HandleDirectory>> =
            Arc::new(Mutex::new(HandleDirectory::with_capacity(MAX_SUBSCRIBERS)));
        let control_block_directory: Arc<RwLock<ControlBlockDirectory>> = Arc::new(RwLock::new(
            ControlBlockDirectory::with_capacity(MAX_SUBSCRIBERS),
        ));
        let white_pages_directory: Arc<RwLock<Directory>> =
            Arc::new(RwLock::new(Directory::with_capacity(MAX_SUBSCRIBERS)));
        let state_directory: Arc<RwLock<StateDirectory>> =
            Arc::new(RwLock::new(StateDirectory::with_capacity(MAX_SUBSCRIBERS)));
        */
        let handle_directory: HandleDirectory = HandleDirectory::with_capacity(MAX_SUBSCRIBERS);
        let control_block_directory: ControlBlockDirectory =
            ControlBlockDirectory::with_capacity(MAX_SUBSCRIBERS);
        let white_pages_directory: Directory = Directory::with_capacity(MAX_SUBSCRIBERS);
        let state_directory: StateDirectory = StateDirectory::with_capacity(MAX_SUBSCRIBERS);
        let mr = Arc::new(RwLock::new(MasterRecord {
            name,
            white_pages_directory,
            control_block_directory,
            handle_directory,
            state_directory,
        }));
        Self {
            //refself: Arc::new(RwLock::new(Self)),
            mr,
        }
    }
    pub fn boot(&mut self) -> Result<(), &str> {
        let default = service::DefaultConditions;
        let mut ams = service::ams::AMS::<DefaultConditions>::new(&self, default);
        let ams_name = ams.service_hub.aid.get_name();
        println!("AMS NAME: {}", ams_name.clone());
        self.mr
            .write()
            .unwrap()
            .white_pages_directory
            //.write()
            //.unwrap()
            .insert(ams_name.clone(), ams.get_aid());

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

        self.mr
            .write()
            .unwrap()
            .handle_directory
            //.lock()
            //.unwrap()
            .insert(ams_name, ams_handle);
        Ok(())
    }

    pub fn add<T>(
        &mut self,
        nickname: String,
        priority: u8,
        stack_size: usize,
        data: T,
    ) -> Result<Agent<T>, &str> {
        let tcb = ControlBlock {
            init: AtomicBool::new(false),
            suspend: AtomicBool::new(false),
            quit: AtomicBool::new(false),
        };
        let hap = self.mr.read().unwrap().name.clone();
        let platform = self.mr.clone();
        //let arc_tcb = Arc::new(tcb);
        let agent_creation = Agent::new(
            nickname.clone(),
            priority,
            stack_size,
            data,
            platform,
            /*
            arc_tcb.clone(),
            self.state_directory.clone(),
            self.white_pages_directory.clone(),*/
        ); //;
        match agent_creation {
            Ok(agent) => {
                self.mr
                    .write()
                    .unwrap()
                    .control_block_directory
                    //.write()
                    //.unwrap()
                    //.insert(nickname.clone(), arc_tcb.clone());
                    .insert(nickname.clone(), tcb);
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
        let agent_handle = spawn(ThreadPriority::Crossplatform(prio), move |_| execute(agent));
        self.mr
            .write()
            .unwrap()
            .handle_directory
            //.lock()
            //.unwrap()
            .insert(nickname, agent_handle);
        Ok(())
    }
}

use std::sync::{
    atomic::AtomicBool,
    mpsc::{Receiver, SyncSender},
    Arc, RwLock,
};
use thread_priority::*;
use {
    agent::{
        behavior::{execute, AgentBehavior, Behavior},
        Agent, ControlBlock,
    },
    deck::Deck,
    entity::{messaging::Message, Description},
    service::{DefaultConditions, Service},
};

pub mod agent;
pub mod deck;
pub mod entity;
pub mod service;
//pub mod organization;

type Priority = ThreadPriorityValue;
type StackSize = usize;
type TX = SyncSender<Message>;
type RX = Receiver<Message>;

pub const DEFAULT_STACK: usize = 8;
pub const MAX_PRIORITY: u8 = 99;
pub const MAX_SUBSCRIBERS: usize = 64;

#[derive(PartialEq, Debug)]
pub enum ErrorCode {
    Disconnected,
    HandleNone,
    ListFull,
    Duplicated,
    NotFound,
    Timeout,
    Invalid,
    InvalidRequest,
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
    pub(crate) name: String,
    pub(crate) ams_aid: Option<Description>,
    pub(crate) deck: Arc<RwLock<Deck>>,
}

impl Platform {
    pub fn new(name: String) -> Self {
        let deck = Arc::new(RwLock::new(Deck::new()));
        Self {
            name,
            ams_aid: None,
            deck,
        }
    }

    pub fn boot(&mut self) -> Result<(), &str> {
        let default = service::DefaultConditions;
        let mut ams = service::ams::AMS::<DefaultConditions>::new(&self, default);
        let ams_name = "AMS".to_string();
        let mut deck = self.deck.write().unwrap();
        deck.white_pages_directory
            .insert(ams_name.clone(), ams.hub.get_aid());
        self.ams_aid = Some(ams.hub.get_aid());
        let ams_handle = std::thread::Builder::new().spawn_with_priority(
            ThreadPriority::Crossplatform(ams.hub.resources.get_priority()),
            move |_| {
                println!("\nBOOTING AMS: {}\n", ams.hub.aid.get_name());
                ams.service_function();
            },
        );
        /*if ams_handle.is_finished() {
            return Err("AMS ended");
        }*/
        if let Ok(handle) = ams_handle {
            deck.handle_directory.insert(ams_name, handle);
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
            active: AtomicBool::new(false),
            wait: AtomicBool::new(false),
            suspend: AtomicBool::new(false),
            quit: AtomicBool::new(false),
        });
        let hap = self.name.clone();
        let deck = self.deck.clone();
        let agent_creation = Agent::new(
            nickname.clone(),
            priority,
            stack_size,
            data,
            deck,
            tcb.clone(),
            hap,
        );
        match agent_creation {
            Ok(mut agent) => {
                self.deck
                    .write()
                    .unwrap()
                    .control_block_directory
                    .insert(nickname.clone(), tcb);
                self.deck
                    .write()
                    .unwrap()
                    .white_pages_directory
                    .insert(nickname, agent.get_aid());
                agent
                    .directory
                    .insert("AMS".to_string(), self.ams_aid.clone().unwrap());
                Ok(agent)
            }
            Err(x) => Err(x),
        }
    }

    pub fn start(&mut self, agent: impl Behavior + Send + 'static) -> Result<(), &str> {
        let nickname = agent.get_nickname();
        let prio = agent.get_resources().get_priority();
        let mut platform = self.deck.write().unwrap();
        let agent_handle = std::thread::Builder::new()
            .spawn_with_priority(ThreadPriority::Crossplatform(prio), move |_| execute(agent));
        if let Ok(handle) = agent_handle {
            platform.handle_directory.insert(nickname, handle);
        } else {
            return Err("Could not launch agent");
        }
        Ok(())
    }
    //COULD ADD PLATFORM FUNCTIONS AND CALL THEM FROM AMS AGENT
}

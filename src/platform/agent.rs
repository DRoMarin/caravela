use crate::platform::{
    deck::{Deck, Directory, SyncType},
    entity::{
        messaging::{Content, Message, MessageType, RequestType},
        Description, Entity, ExecutionResources,
    },
    ErrorCode, StackSize, MAX_PRIORITY, MAX_SUBSCRIBERS, RX,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::AtomicBool,
        mpsc::{sync_channel, RecvError},
        Arc,
        RwLock,
    },
};

pub mod behavior;
pub mod organization;

pub(crate) struct ControlBlock {
    pub active: AtomicBool,
    pub wait: AtomicBool,
    pub suspend: AtomicBool,
    pub quit: AtomicBool,
}

pub struct AgentHub {
    nickname: String,
    hap: String,
    pub aid: Description,
    pub resources: ExecutionResources,
    rx: RX,
    deck: Arc<RwLock<Deck>>,
    pub(crate) tcb: Arc<ControlBlock>,
    //membership: Option<Membership<'a>>,*/
}

pub struct Agent<T> {
    pub hub: AgentHub,
    pub msg: Message,
    pub directory: Directory,
    pub data: T,
    //pub membership,
}

impl AgentHub {
    pub(crate) fn new(
        nickname: String,
        resources: ExecutionResources,
        deck: Arc<RwLock<Deck>>,
        tcb: Arc<ControlBlock>,
        hap: String,
    ) -> Self {
        let (tx, rx) = sync_channel::<Message>(1);
        let name = nickname.clone() + "@" + &hap.clone();
        let aid = Description::new(name, tx, None);
        Self {
            nickname,
            hap,
            aid,
            resources,
            rx,
            deck,
            tcb, //membership,
        }
    }
}

impl<T> Entity for Agent<T> {
    fn get_aid(&self) -> Description {
        self.hub.aid.clone()
    }

    fn get_nickname(&self) -> String {
        self.hub.nickname.clone()
    }

    fn get_hap(&self) -> String {
        self.hub.hap.clone()
    }

    fn get_resources(&self) -> ExecutionResources {
        self.hub.resources.clone()
    }

    //TBD: add block/nonblock parameter
    fn send_to(&mut self, agent: &str) -> Result<(), ErrorCode> {
        if let Some(agent) = self.directory.get(agent) {
            self.send_to_aid(agent.clone())
        } else {
            self.hub
                .deck
                .read()
                .unwrap()
                .send(agent, self.msg.clone(), SyncType::Blocking)
        }
    }

    fn send_to_aid(&mut self, description: Description) -> Result<(), ErrorCode> {
        self.hub
            .deck
            .read()
            .unwrap()
            .send_to_aid(description, self.msg.clone(), SyncType::Blocking)
    }

    fn receive(&mut self) -> Result<MessageType, RecvError> {
        //TBD: could use recv_timeout
        let result = self.hub.rx.recv();
        match result {
            Ok(received_msg) => {
                self.msg = received_msg;
                Ok(self.msg.get_type().unwrap())
            }
            Err(err) => Err(err),
        }
    }
    /*fn receive_timeout(&mut self, timeout: u64) -> MessageType */
}

impl<T> Agent<T> {
    pub(crate) fn new(
        nickname: String,
        priority: u8,
        stack_size: StackSize,
        data: T,
        deck: Arc<RwLock<Deck>>,
        tcb: Arc<ControlBlock>,
        hap: String,
    ) -> Result<Self, &'static str> {
        if priority > (MAX_PRIORITY - 1) {
            return Err("Priority value invalid");
        };
        let msg = Message::new();
        let directory: Directory = HashMap::with_capacity(MAX_SUBSCRIBERS);
        let resources = ExecutionResources::new(priority, stack_size);
        let hub = AgentHub::new(nickname, resources, deck, tcb, hap);
        Ok(Self {
            hub,
            msg,
            directory,
            data,
        })
    }
    pub fn add_contact(&mut self, agent: &str) -> Result<(), ErrorCode> {
        self.msg.set_type(MessageType::Request);
        self.msg
            .set_content(Content::Request(RequestType::Search(agent.to_string())));
        let send_result = self.send_to("AMS");
        if let Err(err) = send_result {
            return Err(err);
        }
        let recv_result = self.receive();
        if let Ok(msg_type) = recv_result {
            match msg_type {
                MessageType::Inform => {
                    if let Some(Content::AID(x)) = self.msg.get_content() {
                        self.add_contact_aid(agent, x)
                    } else {
                        Err(ErrorCode::Invalid)
                    }
                }
                MessageType::Failure => Err(ErrorCode::NotRegistered),

                _ => Err(ErrorCode::Invalid),
            }
        } else {
            Err(ErrorCode::Disconnected)
        }
    }
    pub fn add_contact_aid(
        &mut self,
        nickname: &str,
        description: Description,
    ) -> Result<(), ErrorCode> {
        if self.directory.len().eq(&MAX_SUBSCRIBERS) {
            Err(ErrorCode::ListFull)
        } else if self.directory.contains_key(nickname) {
            Err(ErrorCode::Duplicated)
        } else {
            self.directory.insert(nickname.to_string(), description);
            Ok(())
        }
    }
}

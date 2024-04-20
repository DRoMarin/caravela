use crate::platform::{
    agent::behavior::AgentBehavior,
    deck::{Deck, Directory},
    entity::{
        messaging::{Content, MessageType, RequestType},
        Description, ExecutionResources, Hub,
    },
    ErrorCode, StackSize, MAX_PRIORITY, MAX_SUBSCRIBERS,
};

use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc, RwLock},
};

pub(crate) mod behavior;

pub(crate) struct ControlBlock {
    pub active: AtomicBool,
    pub wait: AtomicBool,
    pub suspend: AtomicBool,
    pub quit: AtomicBool,
}

pub struct Agent<T> {
    pub(crate) hub: Hub,
    //pub msg: Message,
    pub directory: Directory,
    pub(crate) tcb: Arc<ControlBlock>,
    pub data: T,
    //pub membership,
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
        let directory: Directory = HashMap::with_capacity(MAX_SUBSCRIBERS);
        let resources = ExecutionResources::new(priority, stack_size);
        let hub = Hub::new(nickname, resources, deck, hap);
        Ok(Self {
            hub,
            directory,
            tcb,
            data,
        })
    }
    pub fn add_contact(&mut self, agent: &str) -> Result<(), ErrorCode> {
        let msg_type = MessageType::Request;
        let msg_content = Content::Request(RequestType::Search(agent.to_string()));
        self.set_msg(msg_type, msg_content);
        let send_result = self.send_to("AMS");
        if let Err(err) = send_result {
            return Err(err);
        }
        let recv_result = self.receive();
        if let Ok(msg_type) = recv_result {
            match msg_type {
                MessageType::Inform => {
                    if let Content::AID(x) = self.get_msg().get_content() {
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

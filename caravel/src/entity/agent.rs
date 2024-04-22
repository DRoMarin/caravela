pub(crate) mod behavior;

use crate::{
    deck::{Deck, Directory},
    entity::{
        messaging::{Content, Message, MessageType, RequestType},
        Description, ExecutionResources, Hub,
    },
    ErrorCode, StackSize, MAX_PRIORITY, MAX_SUBSCRIBERS,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};

pub(crate) struct ControlBlock {
    pub active: AtomicBool,
    pub wait: AtomicBool,
    pub suspend: AtomicBool,
    pub quit: AtomicBool,
}

pub struct Agent {
    //pub struct Agent<T> {
    pub(crate) hub: Hub,
    //pub msg: Message,
    pub directory: Directory,
    pub(crate) tcb: Arc<ControlBlock>,
    //pub data: T,
    //pub membership,
}

impl Agent {
    //impl<T> Agent<T> {
    pub(crate) fn new(
        nickname: String,
        priority: u8,
        stack_size: StackSize,
        //data: T,
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
        })
    }

    pub fn get_aid(&self) -> Description {
        self.hub.get_aid()
    }

    pub fn get_nickname(&self) -> String {
        self.hub.get_nickname()
    }

    pub fn get_hap(&self) -> String {
        self.hub.get_hap()
    }

    pub fn get_resources(&self) -> ExecutionResources {
        self.hub.get_resources()
    }

    pub fn get_msg(&self) -> Message {
        self.hub.get_msg()
    }

    pub fn set_msg(&mut self, msg_type: MessageType, msg_content: Content) {
        self.hub.set_msg(msg_type, msg_content)
    }

    //TBD: add block/nonblock parameter
    pub fn send_to(&mut self, agent: &str) -> Result<(), ErrorCode> {
        if let Some(agent) = self.directory.get(agent) {
            self.hub.send_to_aid(agent.clone())
        } else {
            self.hub.send_to(agent)
        }
    }

    pub fn send_to_aid(&mut self, description: Description) -> Result<(), ErrorCode> {
        self.hub.send_to_aid(description)
    }

    pub fn receive(&mut self) -> Result<MessageType, ErrorCode> {
        self.hub.receive()
    }

    pub fn add_contact(&mut self, agent: &str) -> Result<(), ErrorCode> {
        let msg_type = MessageType::Request;
        let msg_content = Content::Request(RequestType::Search(agent.to_string()));
        self.set_msg(msg_type, msg_content);
        let send_result = self.send_to("AMS");
        send_result?;
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

    pub fn wait(&self, time: u64) {
        self.tcb.wait.store(true, Ordering::Relaxed);
        let dur = Duration::from_millis(time);
        thread::sleep(dur);
        self.tcb.wait.store(false, Ordering::Relaxed);
    }

    pub(crate) fn set_thread(&mut self) {
        self.hub.set_thread();
    }

    pub(crate) fn init(&mut self) -> bool {
        println!("{}: STARTING", self.get_nickname());
        self.tcb.active.store(true, Ordering::Relaxed);
        true
    }

    pub(crate) fn suspend(&mut self) {
        if self.tcb.suspend.load(Ordering::Relaxed) {
            self.tcb.suspend.store(true, Ordering::Relaxed);
            thread::park();
            self.tcb.suspend.store(false, Ordering::Relaxed);
        }
    }

    pub(crate) fn quit(&self) -> bool {
        self.tcb.quit.load(Ordering::Relaxed)
    }

    pub(crate) fn takedown(&mut self) -> bool {
        let ams = "AMS".to_string();
        let msg_type = MessageType::Request;
        let msg_content = Content::Request(RequestType::Deregister(self.get_nickname()));
        self.set_msg(msg_type, msg_content);
        let _ = self.send_to(&ams);
        true
    }
}

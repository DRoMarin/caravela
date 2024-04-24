pub mod agent;
pub(crate) mod messaging;
pub(crate) mod service;

use crate::{
    deck::{Deck, SyncType},
    {ErrorCode, Priority, StackSize, RX, TX},
};
pub use agent::{behavior::Behavior, Agent};
pub use messaging::{Content, Message, MessageType, RequestType};
use std::{
    fmt::Display,
    sync::{mpsc::sync_channel, Arc, RwLock},
    thread::{current, ThreadId},
};

#[derive(Clone, Debug)] //Default?
pub struct Description {
    nickname: String,
    hap: String,
    tx: TX,
    thread: Option<ThreadId>,
}

#[derive(Clone, Debug, Default)]
pub struct ExecutionResources {
    priority: Priority,
    stack_size: StackSize,
}

#[derive(Debug)]
pub(crate) struct Hub {
    aid: Description,
    resources: ExecutionResources,
    rx: RX,
    deck: Arc<RwLock<Deck>>,
    msg: Message,
}

impl Description {
    fn new(nickname: String, hap: String, tx: TX, thread: Option<ThreadId>) -> Self {
        //Self { name, tx, thread }
        Self {
            nickname,
            hap,
            tx,
            thread,
        }
    }

    pub fn nickname(&self) -> String {
        self.nickname.clone()
        //REFORMAT
    }

    pub fn name(&self) -> String {
        self.to_string()
        //REFORMAT
    }

    pub fn hap(&self) -> String {
        self.hap.clone()
    }

    pub fn address(&self) -> TX {
        self.tx.clone()
    }

    pub fn id(&self) -> Option<ThreadId> {
        self.thread
    }

    pub(crate) fn set_thread(&mut self) {
        self.thread = Some(current().id());
    }
}

impl Display for Description {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.nickname, self.hap)
    }
}

impl ExecutionResources {
    pub(crate) fn new(priority_num: u8, stack_size: StackSize) -> Self {
        let priority = Priority::try_from(priority_num).unwrap();
        Self {
            priority,
            stack_size,
        }
    }

    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn stack_size(&self) -> StackSize {
        self.stack_size
    }
}

impl Display for ExecutionResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prio: u8 = self.priority().into();
        write!(f, "Priority: {}, Stack Size {}", prio, self.stack_size)
    }
}

impl Hub {
    pub(crate) fn new(
        nickname: String,
        resources: ExecutionResources,
        deck: Arc<RwLock<Deck>>,
        hap: String,
    ) -> Self {
        let (tx, rx) = sync_channel::<Message>(1);
        let aid = Description::new(nickname, hap, tx, None);
        let msg = Message::new();
        Self {
            aid,
            resources,
            rx,
            deck,
            msg,
        }
    }

    pub(crate) fn aid(&self) -> Description {
        self.aid.clone()
    }

    /*pub(crate) fn get_nickname(&self) -> String {
        self.nickname.clone()
    }

    pub(crate) fn get_hap(&self) -> String {
        self.hap.clone()
    }*/

    pub(crate) fn resources(&self) -> ExecutionResources {
        self.resources.clone()
    }

    pub(crate) fn msg(&self) -> Message {
        self.msg.clone()
    }

    pub(crate) fn arc_deck(&self) -> Arc<RwLock<Deck>> {
        self.deck.clone()
    }

    pub(crate) fn set_thread(&mut self) {
        self.aid.set_thread();
    }

    pub(crate) fn set_msg(&mut self, msg_type: MessageType, msg_content: Content) {
        self.msg.set_type(msg_type);
        self.msg.set_content(msg_content);
    }

    /*pub(crate) fn send_to(&mut self, agent: &str) -> Result<(), ErrorCode> {
        self.msg.set_sender(self.aid());
        self.deck
            .read()
            .unwrap()
            .send(agent, self.msg.clone(), SyncType::Blocking)
    }*/

    pub(crate) fn send_to_aid(&mut self, description: Description) -> Result<(), ErrorCode> {
        self.msg.set_sender(self.aid());
        self.deck
            .read()
            .unwrap()
            .send_to_aid(description, self.msg.clone(), SyncType::Blocking)
    }

    pub(crate) fn receive(&mut self) -> Result<MessageType, ErrorCode> {
        //TBD: could use recv_timeout
        let result = self.rx.recv();
        match result {
            Ok(received_msg) => {
                self.msg = received_msg;
                Ok(self.msg.message_type())
            }
            Err(err) => Err(ErrorCode::MpscRecv(err)),
        }
    }
}

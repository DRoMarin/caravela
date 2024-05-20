pub mod agent;
pub(crate) mod messaging;
pub(crate) mod service;

use crate::{
    deck::{Deck, SyncType},
    platform::environment::{aid_from_name, aid_from_thread},
    ErrorCode, RX, TX,
};
pub use agent::{behavior::Behavior, Agent};
pub use messaging::{Content, Message, MessageType, RequestType};
use std::{
    fmt::Display,
    hash::{self, Hash},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    thread::ThreadId,
};

#[derive(Clone, Debug, Default)]
pub struct Description {
    nickname: &'static str,
    hap: &'static str,
    tx: Option<TX>,
    thread: Option<ThreadId>,
}

impl Eq for Description {}

impl PartialEq for Description {
    fn eq(&self, other: &Self) -> bool {
        (self.nickname == other.nickname)
            && (self.hap == other.hap)
            && (self.thread == other.thread)
    }
}

impl Hash for Description {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.nickname.hash(state);
        self.hap.hash(state);
        self.thread.hash(state);
    }
}

impl Display for Description {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.nickname, self.hap)
    }
}

impl TryFrom<&str> for Description {
    type Error = ErrorCode;
    fn try_from(value: &str) -> Result<Self, ErrorCode> {
        aid_from_name(value)
    }
}

impl TryFrom<ThreadId> for Description {
    type Error = ErrorCode;
    fn try_from(value: ThreadId) -> Result<Self, Self::Error> {
        aid_from_thread(value)
    }
}

impl Description {
    pub(crate) fn new(nickname: &'static str, hap: &'static str, tx: TX) -> Self {
        Self {
            nickname,
            hap,
            tx: Some(tx),
            thread: None,
        }
    }

    pub fn nickname(&self) -> &str {
        self.nickname
    }

    pub fn name(&self) -> String {
        self.to_string()
    }

    pub fn hap(&self) -> &str {
        self.hap
    }

    pub(crate) fn address(&self) -> Result<TX, ErrorCode> {
        if let Some(address) = &self.tx {
            Ok(address.clone())
        } else {
            Err(ErrorCode::AddressNone)
        }
    }

    pub fn id(&self) -> Option<ThreadId> {
        self.thread
    }

    pub(crate) fn set_thread(&mut self, id: ThreadId) {
        //self.thread = Some(current().id());
        self.thread = Some(id);
    }
}

#[derive(Debug)]
pub(crate) struct Hub {
    aid: Description,
    rx: RX,
    deck: Arc<RwLock<Deck>>,
    msg: Message,
}

impl Hub {
    pub(crate) fn new(rx: RX, deck: Arc<RwLock<Deck>>) -> Self {
        let aid = Description::default();
        let msg = Message::new();
        Self { aid, rx, deck, msg }
    }

    pub(crate) fn aid(&self) -> Description {
        self.aid.clone()
    }

    pub(crate) fn msg(&self) -> Message {
        self.msg.clone()
    }

    pub(crate) fn set_aid(&mut self, aid: Description) {
        self.aid = aid;
    }

    /*pub(crate) fn set_thread(&mut self, id: ThreadId) {
        self.aid.set_thread(id);
    }*/

    pub(crate) fn set_msg(&mut self, msg_type: MessageType, msg_content: Content) {
        self.msg.set_type(msg_type);
        self.msg.set_content(msg_content);
    }

    pub(crate) fn deck_write(&self) -> Result<RwLockWriteGuard<Deck>, ErrorCode> {
        if let Ok(guard) = self.deck.write() {
            Ok(guard)
        } else {
            Err(ErrorCode::PoisonedLock)
        }
    }
    pub(crate) fn deck_read(&self) -> Result<RwLockReadGuard<Deck>, ErrorCode> {
        if let Ok(guard) = self.deck.read() {
            Ok(guard)
        } else {
            Err(ErrorCode::PoisonedLock)
        }
    }

    pub(crate) fn send_to_aid(&mut self, description: &Description) -> Result<(), ErrorCode> {
        self.msg.set_receiver(description.clone());
        self.msg.set_sender(self.aid());
        self.deck_read()?
            .send_msg(self.msg.clone(), SyncType::Blocking)
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

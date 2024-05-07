pub mod agent;
pub(crate) mod messaging;
pub(crate) mod service;

use crate::{
    deck::{Deck, SyncType},
    platform::env::AID_ENV,
    ErrorCode, RX, TX,
};
pub use agent::{behavior::Behavior, Agent};
pub use messaging::{Content, Message, MessageType, RequestType};
use std::{
    fmt::Display,
    hash::{self, Hash},
    sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
    thread::{current, ThreadId},
};

#[derive(Clone, Debug, Default)]
pub struct Description {
    nickname: String,
    hap: String,
    tx: Option<crate::TX>,
    thread: Option<ThreadId>,
}

impl Eq for Description {}

impl PartialEq for Description {
    fn eq(&self, other: &Self) -> bool {
        (self.nickname != other.nickname)
            && (self.hap != other.hap)
            && (self.thread != other.thread)
    }
    fn ne(&self, other: &Self) -> bool {
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

#[derive(Debug, Clone)]
pub struct AidHandler {
    lock: OnceLock<Description>,
}

#[derive(Debug)]
pub(crate) struct Hub {
    aid: Description,
    rx: RX,
    deck: Arc<RwLock<Deck>>,
    msg: Message,
}

impl Description {
    fn new(nickname: String, hap: String, tx: TX, thread_id: ThreadId) -> Self {
        Self {
            nickname,
            hap,
            tx: Some(tx),
            thread: Some(thread_id),
        }
    }

    pub fn nickname(&self) -> String {
        self.nickname.clone()
    }

    pub fn name(&self) -> String {
        self.to_string()
    }

    pub fn hap(&self) -> String {
        self.hap.clone()
    }

    pub(crate) fn address(&self) -> Result<TX, ErrorCode> {
        if let Some(address) = self.tx {
            Ok(address.clone())
        } else {
            Err(ErrorCode::AddressNone)
        }
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

impl TryFrom<&str> for Description {
    type Error = ErrorCode;
    fn try_from(value: &str) -> Result<Self, ErrorCode> {
        if let Some(lock) = AID_ENV.get() {
            if let Ok(aid_env) = lock.read() {
                if let Some(aid) = aid_env.get(value) {
                    Ok(aid.clone())
                } else {
                    Err(ErrorCode::AidHandleNone)
                }
            } else {
                Err(ErrorCode::PoisonedLock)
            }
        } else {
            Err(ErrorCode::UninitEnv)
        }
    }
}

/*impl AidHandler {
    pub const fn new() -> Self {
        Self {
            lock: OnceLock::<Description>::new(),
        }
    }
    pub fn aid(&self) -> Option<&Description> {
        /*match self.lock.get() {
            Some(description) => Some(description.to_owned()),
            None => None,
        }*/
        self.lock.get()
    }
}*/

impl Hub {
    pub(crate) fn new(aid: Description, rx: RX, deck: Arc<RwLock<Deck>>) -> Self {
        let msg = Message::new();
        Self { aid, rx, deck, msg }
    }

    pub(crate) fn aid(&self) -> Description {
        self.aid.clone()
    }

    pub(crate) fn msg(&self) -> Message {
        self.msg.clone()
    }

    pub(crate) fn set_thread(&mut self) {
        self.aid.set_thread();
    }

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

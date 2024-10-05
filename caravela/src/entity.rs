/// Base agent functionality.
pub mod agent;
/// Messaging related types and operations.
pub mod messaging;
/// Service related features.
pub mod service;

use crate::{
    deck::{deck, SyncType},
    ErrorCode, RX, TX,
};
use messaging::{Content, Message, MessageType};
use std::{
    fmt::Display,
    hash::{self, Hash},
    thread::ThreadId,
};

/// Agent Identifier (AID) that is unique to all entities across platforms.
///
/// Each AID has two different parameters:
/// - `Name` which is given with the format `name nickname@hap` and as [`String`].
/// - `Id` which is unique among the process since it identifies the thread executing the entity and it is given as type [`ThreadId`].
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

impl Description {
    pub(crate) fn new(nickname: &'static str, hap: &'static str, tx: TX) -> Self {
        Self {
            nickname,
            hap,
            tx: Some(tx),
            thread: None,
        }
    }

    /// Return a `String` with the full name of the AID. Same result as [`ToString::to_string`].
    pub fn name(&self) -> String {
        self.to_string()
    }

    /// Return a `&str` slice with the nickname of the name; the left side of nickname@hap.
    pub fn nickname(&self) -> &str {
        self.nickname
    }

    /// Return a `&str` slice with name of the Host Agent Platform (HAP) of the name; the right side of nickname@hap.
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

    /// Return an `Option<ThreadId>`: [`Some`] if the Entity is running and [`None`] if not.
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
    //deck: DeckAccess, //Arc<RwLock<Deck>>,
    msg: Message,
}

impl Hub {
    pub(crate) fn new(aid: Description, rx: RX) -> Self {
        //let aid = Description::default();
        let msg = Message::new();
        Self { aid, rx, msg }
    }

    pub(crate) fn aid(&self) -> &Description {
        &self.aid
    }

    pub(crate) fn msg(&self) -> Message {
        self.msg.clone()
    }

    pub(crate) fn set_thread(&mut self, id: ThreadId) {
        self.aid.set_thread(id);
    }

    pub(crate) fn set_msg_receiver(&mut self, aid: Description) {
        self.msg.set_receiver(aid);
    }

    pub(crate) fn set_msg_sender(&mut self, aid: Description) {
        self.msg.set_sender(aid);
    }

    //NAME CHANGE
    pub(crate) fn set_msg(&mut self, msg_type: MessageType, msg_content: Content) {
        self.msg.set_type(msg_type);
        self.msg.set_content(msg_content);
    }

    pub(crate) fn send(&self) -> Result<(), ErrorCode> {
        deck()
            .read()?
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

/// Base agent functionality.
pub mod agent;
/// Messaging related types and operations.
pub mod messaging;
/// Service related features.
pub mod service;

use crate::{ErrorCode, Rx, Tx};
//use messaging::{Content, Message, SendResult, SyncType};
use messaging::{Message, SendResult, SyncType};
use std::{
    fmt::Display,
    hash::{self, Hash},
    sync::mpsc::{SendError, TrySendError},
    thread::ThreadId,
};

/// Agent Identifier (AID) that is unique to all entities across platforms.
///
/// Each AID has two different parameters:
/// - `Name` which is given with the format `name nickname@hap` and as [`String`].
/// - `Id` which is unique among the process since it identifies the thread executing the entity and it is given as type [`ThreadId`].
#[derive(Clone, Debug)]
pub struct Description {
    nickname: &'static str,
    hap: &'static str,
    tx: Tx,
    id: Option<ThreadId>,
}

impl Eq for Description {}

impl PartialEq for Description {
    fn eq(&self, other: &Self) -> bool {
        (self.nickname == other.nickname) && (self.hap == other.hap)
    }
}

impl Hash for Description {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.nickname.hash(state);
        self.hap.hash(state);
        self.id.hash(state);
    }
}

impl Display for Description {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.nickname, self.hap)
    }
}

impl Description {
    pub(crate) fn new(nickname: &'static str, hap: &'static str, tx: Tx) -> Self {
        Self {
            nickname,
            hap,
            tx,
            id: None,
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

    pub(crate) fn address(&self) -> &Tx {
        &self.tx
    }

    /// Return an `Option<ThreadId>`: [`Some`] if the Entity is running and [`None`] if not.
    pub fn id(&self) -> Option<ThreadId> {
        self.id
    }

    pub(crate) fn set_id(&mut self, id: ThreadId) {
        //self.thread = Some(current().id());
        self.id = Some(id);
    }
}

#[derive(Debug)]
pub(crate) struct Hub {
    rx: Rx,
    //deck: DeckAccess, //Arc<RwLock<Deck>>,
    //msg: Option<Message>,
}

impl Hub {
    pub(crate) fn new(rx: Rx) -> Self {
        //let msg = None;
        Self { rx }
        //, msg }
    }

    /*pub(crate) fn msg(&self) -> Message {
        self.msg.clone()
    }*/

    pub(crate) fn send(&self, msg: Message, sync: SyncType) -> Result<(), ErrorCode> {
        caravela_messaging!(
            "{}: Sending {} to {}",
            msg.sender(),
            msg.message_type(),
            msg.receiver()
        );
        //check memberships and roles
        let address = msg.receiver().address().clone();
        let result = match sync {
            SyncType::Blocking => SendResult::Blocking(address.send(msg)),
            SyncType::NonBlocking => SendResult::NonBlocking(address.try_send(msg)),
        };
        match result {
            SendResult::Blocking(Ok(_)) => Ok(()),
            SendResult::NonBlocking(Ok(_)) => Ok(()),
            SendResult::Blocking(Err(SendError(_))) => Err(ErrorCode::Disconnected),
            SendResult::NonBlocking(Err(error)) => match error {
                TrySendError::Disconnected(_) => Err(ErrorCode::Disconnected), //LIST MAY BE OUTDATED
                TrySendError::Full(_) => Err(ErrorCode::ChannelFull),
            },
        }
    }

    pub(crate) fn receive(&self) -> Result<Message, ErrorCode> {
        //TBD: could use recv_timeout
        self.rx.recv().map_err(ErrorCode::MpscRecv)

        //match result {
        //    Ok(received_msg) => {
        //self.msg = Some(received_msg);
        //        Ok(self.msg.message_type())
        //    }
        //    Err(err) => Err(ErrorCode::MpscRecv(err)),
        //}
    }
}

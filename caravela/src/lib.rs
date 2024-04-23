#![warn(missing_docs)]

pub(crate) mod deck;
pub(crate) mod entity;
pub(crate) mod platform;

pub use self::entity::agent;
pub use self::entity::{
    messaging::{Content, Message, MessageType, RequestType},
    Description, ExecutionResources,
};
pub use self::platform::Platform;
use std::sync::mpsc::{Receiver, RecvError, SyncSender};
use thread_priority::*;

pub type Priority = ThreadPriorityValue;
pub type StackSize = usize;
pub(crate) type TX = SyncSender<Message>;
pub(crate) type RX = Receiver<Message>;

pub const DEFAULT_STACK: usize = 8;
pub const MAX_PRIORITY: u8 = 99;
pub const MAX_SUBSCRIBERS: usize = 64;

#[derive(PartialEq, Debug)]
pub enum ErrorCode {
    MpscRecv(RecvError),
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
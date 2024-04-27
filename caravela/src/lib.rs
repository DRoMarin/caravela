//#![warn(missing_docs)]

pub(crate) mod deck;
pub(crate) mod entity;
pub(crate) mod platform;

pub use self::entity::agent;
pub use self::entity::{
    messaging::{Content, Message, MessageType, RequestType},
    Description, ExecutionResources,
};
pub use self::platform::Platform;
use entity::agent::AgentState;
use std::{
    error::Error,
    fmt::Display,
    sync::mpsc::{Receiver, RecvError, SyncSender},
};
use thread_priority::*;

pub type Priority = ThreadPriorityValue;
pub type StackSize = usize;
pub(crate) type TX = SyncSender<Message>;
pub(crate) type RX = Receiver<Message>;

pub const DEFAULT_STACK: usize = 8;
pub const MAX_PRIORITY: u8 = 99;
pub const MAX_SUBSCRIBERS: usize = 64;

#[derive(PartialEq, Debug, Default)]
pub enum ErrorCode {
    AmsBoot,
    AgentLaunch,
    InvalidPriority,
    MpscRecv(RecvError),
    Disconnected,
    ListFull,
    Duplicated,
    NotFound,
    FullChannel,
    InvalidConditions(RequestType),
    InvalidContent,
    InvalidMessageType,
    #[default]
    InvalidRequest,
    InvalidStateChange(AgentState, AgentState),
    NotRegistered,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::AmsBoot => write!(f, "Could not spawn AMS task"),
            ErrorCode::AgentLaunch => write!(f," Could not spawn Agent task"),
            ErrorCode::InvalidPriority => write!(f,"Cannot create agent with this priority value. Priority range corresponds to [0,98]"),
            ErrorCode::MpscRecv(_) => write!(f,"SyncSender was disconnected from this Receiver"),
            ErrorCode::Disconnected => write!(f,"Receiver was disconnected from this SyncSender"),
            ErrorCode::ListFull => write!(f,"Max number of Agents reached"),
            ErrorCode::Duplicated => write!(f,"Agent is already registered"),
            ErrorCode::NotFound => write!(f,"Agent could not be found"),
            ErrorCode::FullChannel => write!(f,"Target Agent channel was full"),
            ErrorCode::InvalidConditions(x) => write!(f,"Conditions not met for: {}",x),
            ErrorCode::InvalidContent => write!(f,"Invalid Content in message"),
            ErrorCode::InvalidMessageType => todo!("Unexpected message received"),
            ErrorCode::InvalidRequest => write!(f,"Unexpected request received"),
            ErrorCode::InvalidStateChange(current, next) => write!(f,"Transtion from {} to {} is not possible",current,next),
            ErrorCode::NotRegistered => write!(f,"Target agent is not registered"),
        }
    }
}

impl Error for ErrorCode {}

//#![warn(missing_docs)]

pub(crate) mod deck;
pub(crate) mod entity;
pub(crate) mod platform;

pub use self::entity::agent;
pub use self::entity::{
    messaging::{Content, Message, MessageType, RequestType},
    Description,
};
pub use self::platform::Platform;
use entity::agent::AgentState;
use std::{
    error::Error,
    fmt::Display,
    sync::mpsc::{Receiver, RecvError, SyncSender},
};
//use thread_priority::*;

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
    AgentStart(thread_priority::Error),
    InvalidPriority(&'static str),
    MpscRecv(RecvError),
    Disconnected,
    ListFull,
    Duplicated,
    NotFound,
    ChannelFull,
    InvalidConditions(RequestType),
    InvalidContent,
    InvalidMessageType,
    #[default]
    InvalidRequest,
    InvalidStateChange(AgentState, AgentState),
    NotRegistered,
    AidHandleNone,
    PoisonedLock,
    UninitEnv,
    AddressNone,
    PoisonedEnvironment,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::AmsBoot => write!(f, "Could not spawn AMS task"),
            ErrorCode::AgentLaunch => write!(f, " Could not spawn agent"),
            ErrorCode::AgentStart(error) => write!(f, " Could not start agent: {:?}", error),
            ErrorCode::InvalidPriority(error) => {
                write!(f, "Could not create agent with this priority:{}", error)
            }
            ErrorCode::MpscRecv(_) => write!(f, "SyncSender was disconnected from this Receiver"),
            ErrorCode::Disconnected => write!(f, "Receiver was disconnected from this SyncSender"),
            ErrorCode::ListFull => write!(f, "Max number of agents reached"),
            ErrorCode::Duplicated => write!(f, "Agent is already present"),
            ErrorCode::NotFound => write!(f, "Agent could not be found"),
            ErrorCode::ChannelFull => write!(f, "Target agent channel was full"),
            ErrorCode::InvalidConditions(x) => write!(f, "Conditions not met for: {}", x),
            ErrorCode::InvalidContent => write!(f, "Invalid content in message"),
            ErrorCode::InvalidMessageType => write!(f, "Unexpected message received"),
            ErrorCode::InvalidRequest => write!(f, "Unexpected request received"),
            ErrorCode::InvalidStateChange(current, next) => {
                write!(f, "Transtion from {} to {} is not possible", current, next)
            }
            ErrorCode::NotRegistered => write!(f, "Target agent is not registered"),
            ErrorCode::AidHandleNone => write!(f, "Target agent has no AID"),
            ErrorCode::PoisonedLock => write!(f, "Platform lock is poisoned"),
            ErrorCode::UninitEnv => write!(f, "Environment has not been initialized yet"),
            ErrorCode::AddressNone => write!(f, "Target agent has not transport address assigned"),
            ErrorCode::PoisonedEnvironment => write!(f, "Environment is poisoned"),
        }
    }
}

impl Error for ErrorCode {}

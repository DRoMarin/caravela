#![warn(missing_docs)]

//! This crate offers a platform to create and run programs based on Multi-Agent Systems
//!  according to the standards set by the Foundation for Intelligent Physical Agents (FIPA).
//!
//! The agents run, communicate and interact following on the threading model included in the [`std::sync`] module,
//!  plus this platform depends on the [`thread_priority`] crate to provide a predictive pre-emptive behavior across agents.
#[macro_use]
pub(crate) mod utils;

pub(crate) mod deck;
pub(crate) mod entity;
pub(crate) mod platform;

pub use {
    entity::agent::behavior,
    entity::{agent, messaging, service, Description},
    platform::Platform,
};

use std::{
    error::Error,
    fmt::Display,
    sync::mpsc::{Receiver, RecvError, SyncSender},
};
use {
    agent::AgentState,
    messaging::{Message, RequestType},
};

/// StackSize defined as platform dependant.
pub type StackSize = usize;
pub(crate) type TX = SyncSender<Message>;
pub(crate) type RX = Receiver<Message>;

/// Default stack value for any given platform.
pub const DEFAULT_STACK: usize = 30000;
/// Maximum priority across all entities.
///  This value is reserved for platform service entities such as the AMS and cannot be used for user defined agents.
pub const MAX_PRIORITY: u8 = 99;
pub(crate) const MAX_SUBSCRIBERS: usize = 64;

/// Different error codes associated with possible platform failures provided to support error handling functionality.
#[derive(PartialEq, Debug, Default)]
pub enum ErrorCode {
    /// Could not spawn the AMS agent.
    AmsBoot,
    /// Could not spawn agent.
    AgentPanic,
    /// Could not start the agent due to a priority error.
    AgentStart(thread_priority::Error),
    /// Could not create agent with the given priority.
    InvalidPriority(&'static str),
    /// The sending half of the channel may have disconnected.
    MpscRecv(RecvError),
    //MpscSyncSend(TrySendError<Message>),
    //MpscSend(SendError<Message>),
    /// The receiving half of the channel may have disconnected.
    Disconnected,
    /// The receiving channel is currently full.
    ChannelFull,
    /// The directory has reached the maximum number of agents.
    ListFull,
    /// The agent is already present.
    Duplicated,
    /// The agent could not be found.
    NotFound,
    /// Conditions not met for a specific request.
    InvalidConditions(RequestType),
    /// Invalid content in message.
    InvalidContent,
    /// Unexpected message for a given protocol.
    InvalidMessageType,
    /// Unexpected request.
    #[default]
    InvalidRequest,
    /// State change not possible.
    InvalidStateChange(AgentState, AgentState),
    /// Target is not registered.
    NotRegistered,
    /// Description not available.
    AidHandleNone,
    /// Function behind lock could not be accessed due to poisoning.
    PoisonedLock,
    /// Enviroment not initiated yet.
    UninitEnv,
    /// Agent adress not assigned yet.
    AddressNone,
    /// Enviroment behind lock is poisoned.
    PoisonedEnvironment,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::AmsBoot => write!(f, "Could not spawn AMS agent"),
            ErrorCode::AgentPanic => write!(f, " Could not spawn agent"),
            ErrorCode::AgentStart(error) => write!(f, " Could not start agent: {:?}", error),
            ErrorCode::InvalidPriority(error) => {
                write!(f, "Could not create agent with this priority:{}", error)
            }
            ErrorCode::MpscRecv(_) => write!(f, "SyncSender was disconnected from this Receiver"),
            ErrorCode::Disconnected => write!(f, "Receiver was disconnected from this SyncSender"),
            ErrorCode::ChannelFull => write!(f, "Target agent channel was full"),
            //ErrorCode::MpscSyncSend(error) => {
            //    write!(f, "Receiver could not accept the message:{}",error)
            //}
            //ErrorCode::MpscSend(_) => write!(f, "Receiver was disconnected from this SyncSender"),
            ErrorCode::ListFull => write!(f, "Max number of agents reached"),
            ErrorCode::Duplicated => write!(f, "Agent is already present"),
            ErrorCode::NotFound => write!(f, "Agent could not be found"),
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

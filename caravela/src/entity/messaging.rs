use crate::entity::Description;
use std::{
    fmt::Display,
    sync::mpsc::{SendError, TrySendError},
};

#[derive(Debug)]
pub(crate) enum SyncType {
    Blocking,
    #[allow(dead_code)]
    NonBlocking, //USE?
}

#[derive(Debug)]
pub(crate) enum SendResult {
    Blocking(Result<(), SendError<Message>>),
    NonBlocking(Result<(), TrySendError<Message>>),
}

/// All communicative acts allowed between agents.
///
/// These are defined by the FIPA00037 standard and are meant to be used with a formal logic language model included in the standard.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum MessageType {
    /// Accept a previously presented proposal to perform some action.
    AcceptProposal,
    /// Agree to perform some action.
    Agree,
    /// Inform than the sender no longer wants the receiver to perform some action.
    Cancel,
    /// Asking for proposal for some action.
    CallForProposal,
    /// Inform the receiver that a proposition that it knowns but its uncertain about is true.
    Confirm,
    /// Inform the receiver that a proposition that it knowns but its uncertain about is false.
    Disconfirm,
    /// Inform the receiver that some action was tried and failed.
    Failure,
    /// Inform the receiver than a proposition is true.
    Inform,
    /// Macro action inform based on what the sender believes.
    InformIf,
    /// Macro action inform to send a descriptor.
    InformRef,
    /// Inform that the content of a message was not understood.
    NotUnderstood,
    /// Request the receiver to resend the message to another agent.
    Propagate,
    /// Present a proposal to the receiver.
    Propose,
    //Proxy,
    /// Ask the receiver if a proposition is true.
    QueryIf,
    /// Ask the receiver for a descriptor of a reference.
    QueryRef,
    /// Refuse to perform an action.
    Refuse,
    //RejectProposal
    /// Request the receiver to perform some action.
    Request,
    /// Request the receiver to perform some action when a proposition becomes true.
    RequestWhen,
    /// Request the receiver to perform some action when a proposition becomes true and each time after it becomes true again.
    RequestWhenever,
    /// Ask a receiver for a descriptor of a refence and each time the reference changes.
    Subscribe,
    #[default]
    /// No message type set. Default value.
    None,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::AcceptProposal => write!(f, "Accept Proposal Message"),
            MessageType::Agree => write!(f, "Agree Message"),
            MessageType::Cancel => write!(f, "Cancel Message"),
            MessageType::CallForProposal => write!(f, "CFP Message"),
            MessageType::Confirm => write!(f, "Confirm Message"),
            MessageType::Disconfirm => write!(f, "Disconfirm Message"),
            MessageType::Failure => write!(f, "Failure Message"),
            MessageType::Inform => write!(f, "Inform Message"),
            MessageType::InformIf => write!(f, "InformIf Message"),
            MessageType::InformRef => write!(f, "InformRef Message"),
            MessageType::NotUnderstood => write!(f, "NotUnderstood Message"),
            MessageType::Propagate => write!(f, "Propagate Message"),
            MessageType::Propose => write!(f, "Propose Message"),
            MessageType::QueryIf => write!(f, "QueryIf Message"),
            MessageType::QueryRef => write!(f, "QueryRef Message"),
            MessageType::Refuse => write!(f, "Refuse Message"),
            MessageType::Request => write!(f, "Request Message"),
            MessageType::RequestWhen => write!(f, "RequestWhen Message"),
            MessageType::RequestWhenever => write!(f, "RequestWhenever Message"),
            MessageType::Subscribe => write!(f, "Subscribe Message"),
            MessageType::None => write!(f, "None"),
        }
    }
}

///Request types supported by different services.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionType {
    /// Request the target to search for an agent.
    Search(Description),
    /// Request the target to modify an agent.
    Modify(Description, String),
    //Modify(Description, ModifyAgent),
    /// Request the target to register an agent.
    Register(Description),
    /// Request the target to deregister an agent.
    Deregister(Description),
    /// Other non-specific action defined by the user.
    Other(String),
}

//impl Display for RequestType {
impl Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::Search(x) => write!(f, "Search Request [{}]", x),
            ActionType::Modify(x, _) => write!(f, "Modify Request[{}]", x),
            ActionType::Register(x) => write!(f, "Registration Request [{}]", x),
            ActionType::Deregister(x) => write!(f, "Deregistration Request [{}]", x),
            ActionType::Other(x) => write!(f, "{}", x),
        }
    }
}

/// Different types of content allowed for messaging.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Content {
    /// Propositions with no specific format.
    Expression(String),
    /// A request to be done.
    Action(ActionType),
    //Request(Description, RequestType),
    //RequestOrg(Performer, RequestType),
    //AMS agent description object.
    //AgentDescription(Description),
}
/// Message object with a payload ([`RequestType`] and [`Content`]) and sender/receiver infromation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    sender: Description,
    receiver: Description,
    message_type: MessageType,
    content: Content,
}

impl Message {
    pub(crate) fn new(
        sender: Description,
        receiver: Description,
        message_type: MessageType,
        content: Content,
    ) -> Self {
        Self {
            sender,
            receiver,
            message_type,
            content,
        }
    }

    /// Retrieve a message's communicative act type.
    pub fn message_type(&self) -> &MessageType {
        &self.message_type
    }

    /// Retrieve a message's contents.
    pub fn content(&self) -> &Content {
        &self.content
    }

    /// Get a reference to the sender's [`Description`]
    pub fn sender(&self) -> &Description {
        &self.sender
    }

    /// Get a reference to the receiver's [`Description`]
    pub fn receiver(&self) -> &Description {
        &self.receiver
    }
}

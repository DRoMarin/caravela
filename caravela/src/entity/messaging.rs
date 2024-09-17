use super::{service::ams::AmsAgentDescription, Description};
use std::fmt::Display;

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
    // NoResponse,
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
            //MessageType::NoResponse => write!(f, "NoResponse Message"),
            MessageType::None => write!(f, "None"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Request types supported by different services.
pub enum RequestType {
    /// Request the receiver to search for an agent.
    Search(Description),
    //Modify(String, Description),
    /// Request the receiver to register an agent.
    Register(Description),
    /// Request the receiver to deregister an agent.
    Deregister(Description),
    /// Request the receiver to suspend an agent. Supported only by the AMS.
    Suspend(Description),
    /// Request the receiver to resume an agent. Supported only by the AMS.
    Resume(Description),
    //Restart(String),
    /// Request the receiver to terminate an agent. Supported only by the AMS.
    Terminate(Description),
    #[default]
    /// No request type set. Default value.
    None,
}

impl Display for RequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestType::None => write!(f, "No request"),
            RequestType::Search(x) => write!(f, "Search Request [{}]", x),
            RequestType::Register(x) => write!(f, "Registration Request [{}]", x),
            RequestType::Deregister(x) => write!(f, "Deregistration Request [{}]", x),
            RequestType::Suspend(x) => write!(f, "Suspension Request [{}]", x),
            RequestType::Resume(x) => write!(f, "Resumption Request [{}]", x),
            RequestType::Terminate(x) => write!(f, "Termination Request [{}]", x),
        }
    }
}

/// Different types of content allowed for messaging.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Content {
    /// Propositions with no specific format.
    Text(String),
    /// A request to be done.
    Request(RequestType),
    /// AMS agent description object.
    AmsAgentDescription(AmsAgentDescription),
    #[default]
    /// No content set. Default value.
    None,
}

/// Message object with a payload ([`RequestType`] and [`Content`]) and sender/receiver infromation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Message {
    sender_aid: Description,
    receiver_aid: Description,
    message_type: MessageType,
    content: Content,
}

impl Message {
    pub(crate) fn new() -> Self {
        //TBD check
        Message::default()
    }

    pub(super) fn set_type(&mut self, msg_type: MessageType) {
        self.message_type = msg_type;
    }

    pub(super) fn set_content(&mut self, msg_content: Content) {
        self.content = msg_content;
    }

    pub(super) fn set_receiver(&mut self, receiver: Description) {
        self.receiver_aid = receiver;
    }

    pub(super) fn set_sender(&mut self, sender: Description) {
        self.sender_aid = sender
    }

    /// Retrieve a message's communicative act type.
    pub fn message_type(&self) -> MessageType {
        self.message_type.clone()
    }

    /// Retrieve a message's contents.
    pub fn content(&self) -> Content {
        self.content.clone()
    }

    /// Get a reference to the sender's [`Description`]
    pub fn sender(&self) -> &Description {
        &self.sender_aid
    }

    /// Get a reference to the receiver's [`Description`]
    pub fn receiver(&self) -> &Description {
        &self.receiver_aid
    }
}

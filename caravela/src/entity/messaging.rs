use super::{service::ams::AmsAgentDescription, Description};
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum MessageType {
    AcceptProposal,
    Agree,
    Cancel,
    CallForProposal,
    Confirm,
    Disconfirm,
    Failure,
    Inform,
    InformIf,
    InformRef,
    NotUnderstood,
    Propagate,
    Propose,
    QueryIf,
    QueryRef,
    Refuse,
    Request,
    RequestWhen,
    RequestWhenever,
    Subscribe,
    NoResponse,
    #[default]
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
            MessageType::NoResponse => write!(f, "NoResponse Message"),
            MessageType::None => write!(f, "None"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum RequestType {
    #[default]
    None,
    Search(Description),
    //Modify(String, Description),
    Register(Description),
    Deregister(Description),
    Suspend(Description),
    Resume(Description),
    //Restart(String),
    Terminate(Description),
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

#[derive(Clone, Debug, Default)]
pub enum Content {
    #[default]
    None,
    Text(String),
    Request(RequestType),
    AmsAgentDescription(AmsAgentDescription),
}

#[derive(Clone, Debug, Default)]
pub struct Message {
    sender_aid: Description,
    receiver_aid: Description,
    message_type: MessageType,
    content: Content,
}

impl Message {
    pub(crate) fn new() -> Self {
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

    pub fn message_type(&self) -> MessageType {
        self.message_type.clone()
    }

    pub fn content(&self) -> Content {
        self.content.clone()
    }

    pub fn sender(&self) -> &Description {
        &self.sender_aid
    }

    pub fn receiver(&self) -> &Description {
        &self.receiver_aid
    }
}

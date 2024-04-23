use super::Description;

#[derive(Clone, PartialEq)]
pub enum MessageType {
    AcceptProposal,
    Agree,
    Cancel,
    CFP,
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
    None,
}

#[derive(Clone)]
pub enum RequestType {
    None,
    Search(String),
    //Modify(String, Description),
    Register(String),
    //Register(String, Description),
    Deregister(String),
    Suspend(String),
    Resume(String),
    //Restart(String),
    Terminate(String),
}

#[derive(Clone)]
pub enum Content {
    None,
    Text(String),
    Request(RequestType),
    AID(Description),
}

#[derive(Clone)]
pub struct Message {
    sender_aid: Option<Description>,
    receiver_aid: Option<Description>,
    message_type: MessageType,
    content: Content,
}

impl Message {
    pub(crate) fn new() -> Self {
        Message {
            sender_aid: None,
            receiver_aid: None,
            message_type: MessageType::None,
            content: Content::None,
        }
    }

    pub(super) fn set_type(&mut self, msg_type: MessageType) {
        self.message_type = msg_type;
    }

    pub(super) fn set_content(&mut self, msg_content: Content) {
        self.content = msg_content;
    }
    #[allow(dead_code)]
    pub(super) fn set_receiver(&mut self, receiver: Description) {
        self.receiver_aid = Some(receiver);
    }

    pub(super) fn set_sender(&mut self, sender: Description) {
        self.sender_aid = Some(sender)
    }

    pub fn message_type(&self) -> MessageType {
        self.message_type.clone()
    }

    pub fn content(&self) -> Content {
        self.content.clone()
    }

    pub fn sender(&self) -> Option<Description> {
        self.sender_aid.clone()
    }

    pub fn receiver(&self) -> Option<Description> {
        self.receiver_aid.clone()
    }
}

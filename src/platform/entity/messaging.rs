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
    message_type: Option<MessageType>,
    content: Option<Content>,
}

impl Message {
    pub(crate) fn new() -> Self {
        Message {
            sender_aid: None,
            receiver_aid: None,
            message_type: None,
            content: None,
        }
    }
    pub fn set_type(&mut self, msg_type: MessageType) {
        self.message_type = Some(msg_type);
    }
    pub fn set_content(&mut self, msg_content: Content) {
        self.content = Some(msg_content);
    }
    pub fn set_receiver(&mut self, receiver: Description) {
        self.receiver_aid = Some(receiver);
    }
    pub fn set_sender(&mut self, sender: Description) {
        self.sender_aid = Some(sender)
    }
    pub fn get_type(&mut self) -> Option<MessageType> {
        self.message_type.clone()
    }
    pub fn get_content(&mut self) -> Option<Content> {
        self.content.clone()
    }
    pub fn get_sender(&mut self) -> Option<Description> {
        self.sender_aid.clone()
    }
    pub fn get_receiver(&mut self) -> Option<Description> {
        self.receiver_aid.clone()
    }
}

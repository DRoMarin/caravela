//NEED TO ADD TRANSPORT SERVICE FUNCTIONALITY THAT WILL MANAGE ALL MPSC CHANNELS PER AGENT
//NEED TO DEFINE WHAT WILL HOLD THE LIST OF CONTACTS

use crate::platform::{RX, MAX_SUBSCRIBERS};

use super::{Description, TX};

#[derive(Clone, Copy)]
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
    RequestWhen,
    RequestWhenever,
    Subscribe,
    NoResponse,
}

#[derive(Clone)]
pub enum ContentType {
    Text(String),
    Address(TX),
}

#[derive(Clone)]
pub struct Message {
    sender_aid: Option<Description>,
    receiver_aid: Option<Description>,
    message_type: Option<MessageType>,
    content: Option<ContentType>,
}

pub struct MessageDispatcher {
    message: Message,
    rx: RX,
    contact_list: Vec<Description>, 
}

impl Message {
    fn new() -> Self {
        Message {
            sender_aid: None,
            receiver_aid: None,
            message_type: None,
            content: None,
        }
    }
    fn set_type(&mut self, msg_type: MessageType) {
        self.message_type = Some(msg_type);
    }
    fn set_content(&mut self, msg_content: ContentType) {
        self.content = Some(msg_content);
    }
    fn set_receiver(&mut self, receiver: Description) {
        self.receiver_aid = Some(receiver);
    }
    fn get_type(&self) -> Option<MessageType> {
        self.message_type
    }
    fn get_content(&self) -> Option<ContentType> {
        self.content.clone()
    }
    fn get_sender(&self) -> Option<Description> {
        self.sender_aid.clone()
    }
}

impl MessageDispatcher {
    pub(crate) fn new(rx: RX) -> Self {
        let message = Message::new();
        let contact_list = Vec::with_capacity(MAX_SUBSCRIBERS);
        Self { message, rx, contact_list}
    }
    //ADD DISPATCH FUNCTIONS
    //CONTACT LIST FUNCTIONS
}

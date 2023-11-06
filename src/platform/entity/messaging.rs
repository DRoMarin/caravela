//NEED TO ADD TRANSPORT SERVICE FUNCTIONALITY THAT WILL MANAGE ALL MPSC CHANNELS PER AGENT
//NEED TO DEFINE WHAT WILL HOLD THE LIST OF CONTACTS

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
    RequestWhen,
    RequestWhenever,
    Subscribe,
    NoResponse,
}

#[derive(Clone)]
pub enum Content {
    Text(String),
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
    fn set_type(&mut self, msg_type: MessageType) {
        self.message_type = Some(msg_type);
    }
    fn set_content(&mut self, msg_content: Content) {
        self.content = Some(msg_content);
    }
    fn set_receiver(&mut self, receiver: Description) {
        self.receiver_aid = Some(receiver);
    }
    fn set_sender(&mut self, sender: Description) {
        self.sender_aid = Some(sender)
    }

    fn get_type(&self) -> &Option<MessageType> {
        &self.message_type
    }
    fn get_content(&self) -> &Option<Content> {
        &self.content
    }
    fn get_sender(&self) -> &Option<Description> {
        &self.sender_aid
    }
    fn get_receiver(&self) -> &Option<Description> {
        &self.receiver_aid
    }
}

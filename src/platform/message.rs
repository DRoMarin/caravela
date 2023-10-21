//NEED TO ADD TRANSPORT SERVICE FUNCTIONALITY THAT WILL MANAGE ALL MPSC CHANNELS PER AGENT
//NEED TO DEFINE WHAT WILL HOLD THE LIST OF CONTACTS

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

#[derive(Clone, Copy)]
pub struct Message {
    sender_aid: Option<std::thread::ThreadId>,
    receiver_aid: Option<std::thread::ThreadId>,
    message_type: Option<MessageType>,
    content: Option<&'static str>,
}

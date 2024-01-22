use crate::platform::{
    entity::{
        messaging::{Content, Message, MessageType, RequestType},
        Description, Entity, ExecutionResources,
    },
    Directory, ErrorCode, SharedRecord, StackSize, MAX_PRIORITY, MAX_SUBSCRIBERS, RX,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::AtomicBool,
        mpsc::{sync_channel, TrySendError},
        Arc, Barrier, RwLock,
    },
};

pub mod behavior;
pub mod organization;

pub(crate) struct ControlBlock {
    pub init: Barrier,
    pub suspend: AtomicBool,
    pub quit: AtomicBool,
}

pub struct AgentHub {
    nickname: String,
    hap: String,
    pub aid: Description,
    pub resources: ExecutionResources,
    rx: RX,
    platform: Arc<RwLock<SharedRecord>>,
    pub(crate) tcb: Arc<ControlBlock>,
    //membership: Option<Membership<'a>>,*/
}

pub struct Agent<T> {
    pub hub: AgentHub,
    pub msg: Message,
    pub directory: Directory,
    pub data: T,
    //pub membership,
}

impl AgentHub {
    pub(crate) fn new(
        nickname: String,
        resources: ExecutionResources,
        platform: Arc<RwLock<SharedRecord>>,
        tcb: Arc<ControlBlock>,
    ) -> Self {
        let (tx, rx) = sync_channel::<Message>(1);
        let hap = platform.read().unwrap().name.clone();
        let name = nickname.clone() + "@" + &hap.clone();
        let aid = Description::new(name, tx, None);
        Self {
            nickname,
            hap,
            aid,
            resources,
            rx,
            platform,
            tcb, //membership,
        }
    }
}

impl<T> Entity for Agent<T> {
    fn get_aid(&self) -> Description {
        self.hub.aid.clone()
    }
    fn get_nickname(&self) -> String {
        self.hub.nickname.clone()
    }
    fn get_hap(&self) -> String {
        self.hub.hap.clone()
    }
    fn get_resources(&self) -> ExecutionResources {
        self.hub.resources.clone()
    }
    fn send_to(&mut self, agent: &str) -> ErrorCode {
        let receiver = match self.directory.get(agent) {
            Some(x) => x.clone(),
            None => {
                let msg_bkp = self.msg.clone();
                self.msg.set_type(MessageType::Request);
                self.msg
                    .set_content(Content::Request(RequestType::Search(agent.to_string())));
                self.send_to("AMS");
                let request = self.receive();
                if request == MessageType::Inform {
                    if let Some(Content::AID(x)) = self.msg.get_content() {
                        self.msg = msg_bkp;
                        x
                    } else {
                        return ErrorCode::Invalid;
                    }
                } else if request == MessageType::Failure {
                    return ErrorCode::NotRegistered;
                } else {
                    return ErrorCode::Invalid; //<-----might need to change
                }
            }
        };
        self.send_to_aid(receiver)
    }
    fn send_to_aid(&mut self, description: Description) -> ErrorCode {
        let address = description.get_address().clone();
        self.msg.set_sender(self.hub.aid.clone());
        self.msg.set_receiver(description);
        let result = address.try_send(self.msg.clone());
        let error_code = match result {
            Ok(_) => ErrorCode::NoError,
            Err(error) => match error {
                TrySendError::Full(_) => ErrorCode::Timeout,
                TrySendError::Disconnected(_) => ErrorCode::NotRegistered, //LIST MAY BE OUTDATED
            },
        };
        error_code
    }
    fn receive(&mut self) -> MessageType {
        //could use recv_timeout
        let result = self.hub.rx.recv();
        let msg_type = match result {
            Ok(received_msg) => {
                self.msg = received_msg;
                self.msg.get_type().clone().unwrap()
            }
            Err(_) => MessageType::NoResponse,
        }; //could handle Err incase of disconnection
        msg_type
    }
    /*fn receive_timeout(&mut self, timeout: u64) -> MessageType */
}

impl<T> Agent<T> {
    pub(crate) fn new(
        nickname: String,
        priority: u8,
        stack_size: StackSize,
        data: T,
        platform: Arc<RwLock<SharedRecord>>,
        tcb: Arc<ControlBlock>,
    ) -> Result<Self, &'static str> {
        if priority > (MAX_PRIORITY - 1) {
            return Err("Priority value invalid");
        };
        let msg = Message::new();
        let directory: Directory = HashMap::with_capacity(MAX_SUBSCRIBERS);
        let resources = ExecutionResources::new(priority, stack_size);
        let hub = AgentHub::new(nickname, resources, platform, tcb);
        Ok(Self {
            hub,
            msg,
            directory,
            data,
        })
    }
    pub fn add_contact(&mut self, agent: &str) -> ErrorCode {
        self.msg.set_type(MessageType::Request);
        self.msg
            .set_content(Content::Request(RequestType::Search(agent.to_string())));
        println!("{} ASKING AMS", self.get_nickname());
        self.send_to("AMS");
        let search_result = self.receive();
        println!("{} GOT MESSAGE FROM AMS", self.get_nickname());
        if search_result == MessageType::Inform {
            if let Some(Content::AID(x)) = self.msg.get_content() {
                self.add_contact_aid(agent, x);
                //println!("{} SUCCESSFULLY ADDED", self.get_nickname());
                return ErrorCode::NoError;
            } else {
                return ErrorCode::Invalid;
            }
        } else if search_result == MessageType::Failure {
            //println!("{} TARGET IS NOT REGISTERED", self.get_nickname());
            return ErrorCode::NotRegistered;
        } else {
            return ErrorCode::Invalid;
        }
    }
    pub fn add_contact_aid(&mut self, nickname: &str, description: Description) -> ErrorCode {
        if self.directory.len().eq(&MAX_SUBSCRIBERS) {
            return ErrorCode::ListFull;
        } else if self.directory.contains_key(nickname) {
            return ErrorCode::Duplicated;
        } else {
            self.directory.insert(nickname.to_string(), description);
            ErrorCode::NoError
        }
    }
}

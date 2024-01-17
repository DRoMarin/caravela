use crate::platform::{
    entity::{
        messaging::{Content, Message, MessageType, RequestType},
        Description, Entity, ExecutionResources,
    },
    Directory, ErrorCode, MasterRecord, StackSize, MAX_PRIORITY, MAX_SUBSCRIBERS, RX,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::AtomicBool,
        mpsc::{sync_channel, TrySendError},
        Arc, RwLock,
    },
};

pub mod behavior;
pub mod organization;

pub(crate) struct ControlBlock {
    pub init: AtomicBool,
    pub suspend: AtomicBool,
    pub quit: AtomicBool,
}

pub struct AgentHub {
    nickname: String,
    hap: String,
    pub aid: Description,
    pub resources: ExecutionResources,
    rx: RX,
    pub msg: Message,
    pub directory: Directory,
    platform: Arc<RwLock<MasterRecord>>,
    //membership: Option<Membership<'a>>,*/
}

pub struct Agent<T> {
    pub hub: AgentHub,
    pub data: T,
    //pub membership,
}

impl AgentHub {
    pub(crate) fn new(
        nickname: String,
        resources: ExecutionResources,
        platform: Arc<RwLock<MasterRecord>>,
    ) -> Self {
        let (tx, rx) = sync_channel::<Message>(1);
        let hap = platform.read().unwrap().name.clone();
        let name = nickname.clone() + "@" + &hap.clone();
        let aid = Description::new(name, tx, None);
        let msg = Message::new();
        //format name, set ID, set channel and set HAP

        let directory: Directory = HashMap::with_capacity(MAX_SUBSCRIBERS);
        Self {
            nickname,
            hap,
            aid,
            resources,
            rx,
            msg,
            directory,
            platform,
            //membership,
        }
    }
    //manage contact list pending
}

impl<T> Entity for Agent<T> {
    //Getters
    fn get_aid(&self) -> Description {
        self.hub.aid.clone()
    }
    fn get_nickname(&self) -> String {
        self.hub.nickname.clone()
    }
    fn get_resources(&self) -> ExecutionResources {
        self.hub.resources.clone()
    }
    fn send_to(&mut self, agent: &str) -> ErrorCode {
        //println!("AMS NAME: {}", agent);
        let receiver = match self.hub.directory.get(agent) {
            Some(x) => x.clone(),
            None => {
                let msg_bkp = self.hub.msg.clone();
                self.hub.msg.set_type(MessageType::Request);
                self.hub
                    .msg
                    .set_content(Content::Request(RequestType::Search(agent.to_string())));
                self.send_to("AMS");
                let request = self.receive();
                if request == MessageType::Inform {
                    if let Some(Content::AID(x)) = self.hub.msg.get_content() {
                        self.hub.msg = msg_bkp;
                        x
                    } else {
                        return ErrorCode::Invalid;
                    }
                } else {
                    return ErrorCode::NotRegistered;
                }
            }
        };
        self.send_to_aid(receiver)
    }
    fn send_to_aid(&mut self, description: Description) -> ErrorCode {
        let address = description.get_address().clone();
        self.hub.msg.set_sender(self.hub.aid.clone());
        self.hub.msg.set_receiver(description);
        let result = address.try_send(self.hub.msg.clone());
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
                self.hub.msg = received_msg;
                self.hub.msg.get_type().clone().unwrap()
            }
            Err(_) => MessageType::NoResponse,
        }; //could handle Err incase of disconnection
        msg_type
    }
}

impl<T> Agent<T> {
    pub(crate) fn new(
        nickname: String,
        priority: u8,
        stack_size: StackSize,
        data: T,
        platform: Arc<RwLock<MasterRecord>>,
    ) -> Result<Self, &'static str> {
        if priority > (MAX_PRIORITY - 1) {
            return Err("Priority value invalid");
        };
        let resources = ExecutionResources::new(priority, stack_size);
        let hub = AgentHub::new(nickname, resources, platform);
        //let hub = AgentHub::new(nickname, resources, hap, tcb, state_directory, white_pages);
        Ok(Self { hub, data })
    }
}

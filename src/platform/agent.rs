use crate::platform::{
    entity::{
        messaging::{Message, MessageType},
        Description, Entity, ExecutionResources,
    },
    Directory, ErrorCode, StackSize, StateDirectory, MAX_PRIORITY, MAX_SUBSCRIBERS, RX,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::AtomicBool,
        mpsc::{sync_channel, TrySendError},
        Arc, Mutex, RwLock,
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
    pub(crate) control_block: Arc<ControlBlock>,
    pub(crate) state_directory: Arc<RwLock<StateDirectory>>,
    pub(crate) white_pages: Arc<RwLock<Directory>>,
    //membership: Option<Membership<'a>>,
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
        platform: String,
        control_block: Arc<ControlBlock>,
        state_directory: Arc<RwLock<StateDirectory>>,
        white_pages: Arc<RwLock<Directory>>,
    ) -> Self {
        let (tx, rx) = sync_channel::<Message>(1);
        let name = nickname.clone() + "@" + &platform;
        let hap = platform;
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
            control_block,
            state_directory,
            white_pages,
            //membership,
        }
    }
    /*pub(crate) fn get_tcb(&self) -> Arc<ControlBlock> {
        self.control_block.clone()
    }
    pub(crate) fn set_state_directory(&mut self, state_directory: Arc<Mutex<StateDirectory>>) {
        self.state_directory = Some(state_directory);
    }
    pub(crate) fn get_white_pages_directory(
        &mut self,
        white_pages_directory: Arc<RwLock<Directory>>,
    ) {
        self.white_pages = Some(white_pages_directory);
    }*/
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
                if let Some(description) = self.hub.white_pages.as_ref().read().unwrap().get(agent)
                {
                    description.clone()
                } else {
                    return ErrorCode::NotRegistered;
                }
            }
        };
        self.hub.msg.set_sender(self.hub.aid.clone());
        let address = receiver.get_address().clone();
        self.hub.msg.set_receiver(receiver);
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
        hap: String,
        data: T,
        tcb: Arc<ControlBlock>,
        state_directory: Arc<RwLock<StateDirectory>>,
        white_pages: Arc<RwLock<Directory>>,
    ) -> Result<Self, &'static str> {
        if priority > (MAX_PRIORITY - 1) {
            return Err("Priority value invalid");
        };
        let resources = ExecutionResources::new(priority, stack_size);
        let hub = AgentHub::new(nickname, resources, hap, tcb, state_directory, white_pages);
        Ok(Self { hub, data })
    }
}

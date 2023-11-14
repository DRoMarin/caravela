pub mod behavior;
pub mod organization;

use std::{
    collections::HashMap,
    sync::{
        atomic::AtomicBool,
        mpsc::{sync_channel, TrySendError},
        Arc, Mutex, RwLock,
    },
    thread::{current, Thread},
};

use crate::platform::{
    entity::{messaging::Message, Description, Entity, ExecutionResources},
    Directory, StackSize, StateDirectory, ThreadPriority, MAX_SUBSCRIBERS, RX,
};

use super::{entity::messaging::MessageType, ErrorCode};

pub(crate) struct ControlBlock {
    pub init: AtomicBool,
    pub suspend: AtomicBool,
    pub quit: AtomicBool,
}

pub struct AgentHub {
    nickname: String,
    pub aid: Description,
    pub resources: ExecutionResources,
    rx: RX,
    pub msg: Message,
    pub directory: Directory,
    pub(crate) control_block: Option<ControlBlock>,
    pub(crate) state_directory: Option<Arc<Mutex<StateDirectory>>>,
    pub(crate) white_pages: Option<Arc<RwLock<Directory>>>,
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
        thread: Thread,
        hap: &str,
        control_block: Option<ControlBlock>,
    ) -> Self {
        let (tx, rx) = sync_channel::<Message>(1);
        let name = nickname.clone() + "@" + hap;
        let aid = Description::new(name, tx, thread);
        let msg = Message::new();
        //format name, set ID, set channel and set HAP

        let directory: Directory = HashMap::with_capacity(MAX_SUBSCRIBERS);
        Self {
            nickname,
            aid,
            resources,
            rx,
            msg,
            directory,
            control_block: None,
            state_directory: None,
            white_pages: None,
            //membership,
        }
    }
    pub(crate) fn set_tcb(&mut self, tcb: ControlBlock) {
        self.control_block = Some(tcb);
    }
    pub(crate) fn set_state_directory(&mut self, state_directory: Arc<Mutex<StateDirectory>>) {
        self.state_directory = Some(state_directory);
    }
    pub(crate) fn white_pages_directory(&mut self, state_directory: Arc<Mutex<StateDirectory>>) {
        self.state_directory = Some(state_directory);
    }
}

impl Entity for AgentHub {
    //Getters
    fn get_aid(&self) -> Description {
        self.aid.clone()
    }
    fn get_nickname(&self) -> String {
        self.nickname.clone()
    }
    fn get_resources(&self) -> ExecutionResources {
        self.resources.clone()
    }
    fn send_to(&mut self, agent: &str) -> ErrorCode {
        let receiver = match self.directory.get(agent) {
            Some(x) => x.clone(),
            None => {
                if let Some(description) = self
                    .white_pages
                    .as_ref()
                    .unwrap()
                    .read()
                    .unwrap()
                    .get(agent)
                {
                    description.clone()
                } else {
                    return ErrorCode::NotRegistered;
                }
            }
        };
        //TBD: CHECK ORG VALUES OF RECEIVER
        self.msg.set_sender(self.aid.clone());
        let address = receiver.get_address().clone();
        self.msg.set_receiver(receiver);
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
        let result = self.rx.recv();
        let msg_type = match result {
            Ok(received_msg) => {
                self.msg = received_msg;
                self.msg.get_type().clone().unwrap()
            }
            Err(_) => MessageType::NoResponse,
        }; //could handle Err incase of disconnection
        msg_type
    }

    //Messaging needed
}

impl<T> Agent<T> {
    pub fn new(
        nickname: String,
        priority: ThreadPriority,
        stack_size: StackSize,
        hap: &str,
        data: T,
    ) -> Self {
        let id = current();
        let resources = ExecutionResources::new(priority, stack_size);
        let hub = AgentHub::new(nickname, resources, id, hap, None);
        Self { hub, data }
    }
}
pub mod behavior;
pub mod organization;

use core::time;
use std::{
    collections::HashMap,
    result,
    sync::{
        atomic::AtomicBool,
        mpsc::{sync_channel, TrySendError},
        Arc, Mutex, RwLock,
    },
    thread::{self, current, Thread},
    time::Duration,
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
        self.msg.set_sender(self.aid.clone());

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
    /*
        fn send_to_with_timeout(&mut self, agent: &str, timeout: u64) -> ErrorCode {
            let result = self.send_to(agent);
            if timeout <= 0 {
                return result;
            }
            if result == ErrorCode::Timeout {
                let dur = Duration::from_millis(timeout);
                thread::sleep(dur);
                return self.send_to(agent);
            } else {
                return result;
            };
        }
    */

    fn receive(&mut self) -> MessageType {
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

/*mod private_task_control {
    //THIS SHOULD PROVIDE
    use super::Agent;
    pub(crate) trait TaskControl {
        //TBD
        fn suspend(&self) {}
        fn wait(&self, time: i32) {}
        fn execute(&self) {}
    }
    impl<T> TaskControl for Agent<T> {}
}*/
//Example
/*
struct A {}

impl<A> Behavior for Agent<A> {
    fn action(&mut self) {
    }
}
*/

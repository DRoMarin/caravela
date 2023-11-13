pub mod messaging;

use std::thread::{current, Thread};

use crate::platform::{StackSize, ThreadPriority, TX, ErrorCode};

use self::messaging::{Message, MessageType};

#[derive(Clone)]
pub struct Description {
    name: String,
    tx: TX,
    pub(crate) thread: Thread,
}

#[derive(Clone)]
pub struct ExecutionResources {
    priority: ThreadPriority, //TBD
    stack_size: StackSize,
    //Behavior?
}

impl Description {
    pub fn new(name: String, tx: TX, thread: Thread) -> Self {
        Self { name, tx, thread }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_address(&self) -> TX {
        self.tx.clone()
    }
    pub fn get_id(&self) -> Thread {
        self.thread.clone()
    }
    pub(crate) fn set_thread(&mut self) {
        self.thread = current();
    }
}

impl ExecutionResources {
    pub fn new(priority: ThreadPriority, stack_size: StackSize) -> Self {
        Self {
            priority,
            stack_size,
        }
    }
    pub fn get_priority(&self) -> ThreadPriority {
        self.priority
    }
    pub fn get_stack_size(&self) -> usize {
        self.stack_size
    }
}
pub trait Entity {
    //this trait will define top level gets and actions like recv and send msg
    fn get_aid(&self) -> Description;
    fn get_nickname(&self) -> String;
    fn get_resources(&self) -> ExecutionResources;
    fn send_to(&mut self, agent: &str) -> ErrorCode;
    //fn send_to_with_timeout(&mut self, agent: &str, timeout: u64) -> ErrorCode;
    //fn send_to_all(&self) -> ErrorCode;
    fn receive(&mut self) -> MessageType;
    //fn get_thread_id(&self) -> Option<ID>;
    //MESSAGING GOES HERE
}

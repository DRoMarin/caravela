use crate::platform::{ErrorCode, Priority, StackSize, TX};
use messaging::MessageType;
use std::{
    sync::mpsc::RecvError,
    thread::{current, ThreadId},
};

pub mod messaging;

#[derive(Clone)]
pub struct Description {
    name: String,
    tx: TX, 
    pub(crate) thread: Option<ThreadId>,
}

#[derive(Clone)]
pub struct ExecutionResources {
    priority: Priority, 
    stack_size: StackSize,
}

impl Description {
    pub fn new(name: String, tx: TX, thread: Option<ThreadId>) -> Self {
        Self { name, tx, thread }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    
    pub fn get_address(&self) -> TX {
        self.tx.clone()
    }
    
    pub fn get_id(&self) -> Option<ThreadId> {
        self.thread.clone()
    }
    
    pub(crate) fn set_thread(&mut self) {
        self.thread = Some(current().id());
    }
}

impl ExecutionResources {
    pub fn new(priority_num: u8, stack_size: StackSize) -> Self {
        let priority = Priority::try_from(priority_num).unwrap();
        Self {
            priority,
            stack_size,
        }
    }

    pub fn get_priority(&self) -> Priority {
        self.priority.clone()
    }

    pub fn get_priority_value(&self) -> u8 {
        self.priority.into()
    }

    pub fn get_stack_size(&self) -> usize {
        self.stack_size
    }
}
pub trait Entity {
    //this trait will define top level gets and actions like recv and send msg
    fn get_aid(&self) -> Description;
    fn get_nickname(&self) -> String;
    fn get_hap(&self) -> String;
    fn get_resources(&self) -> ExecutionResources;
    fn send_to(&mut self, agent: &str) -> Result<(), ErrorCode>;
    fn send_to_aid(&mut self, description: Description) -> Result<(), ErrorCode>;
    //fn send_to_with_timeout(&mut self, agent: &str, timeout: u64) -> ErrorCode;
    //fn send_to_all(&self) -> ErrorCode;
    fn receive(&mut self) -> Result<MessageType, RecvError>;
    //fn get_thread_id(&self) -> Option<ID>;
}

use crate::platform::{ErrorCode, Priority, StackSize, TX};
use messaging::MessageType;
use std::thread::{current, ThreadId};

pub mod messaging;

/*#[derive(Clone)]
pub struct Description {
    name: String,
    tx: TX, //could remove due to developments on deck structure
    pub(crate) thread: Option<Thread>,
}*/

//REPLACE FOR THIS
#[derive(Clone)]
pub struct Description {
    name: String,
    tx: TX, //could remove due to developments on deck structure
    pub(crate) thread: Option<ThreadId>,
}

#[derive(Clone)]
pub struct ExecutionResources {
    priority: Priority, //TBD
    stack_size: StackSize,
    //Behavior?
}

impl Description {
    //pub fn new(name: String, tx: TX, thread: Option<Thread>) -> Self {
    pub fn new(name: String, tx: TX, thread: Option<ThreadId>) -> Self {
        //pub fn new(name: String, thread: Option<ThreadId>) -> Self {
        Self { name, tx, thread }
        //Self { name, thread }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_address(&self) -> TX {
        self.tx.clone()
    }
    //pub fn get_id(&self) -> Option<Thread> {
    pub fn get_id(&self) -> Option<ThreadId> {
        self.thread.clone()
    }
    pub(crate) fn set_thread(&mut self) {
        //self.thread = Some(current());
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
    fn send_to(&mut self, agent: &str) -> ErrorCode;
    fn send_to_aid(&mut self, description: Description) -> ErrorCode;
    //fn send_to_with_timeout(&mut self, agent: &str, timeout: u64) -> ErrorCode;
    //fn send_to_all(&self) -> ErrorCode;
    fn receive(&mut self) -> MessageType;
    //fn get_thread_id(&self) -> Option<ID>;
    //MESSAGING GOES HERE
}

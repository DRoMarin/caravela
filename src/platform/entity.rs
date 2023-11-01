pub mod messaging;

use crate::platform::{StackSize, ThreadPriority, ID, RX, TX};

#[derive(Clone)]
pub struct Description {
    id: String,
    channel_tx: Option<TX>,
}

#[derive(Clone)]
pub struct ExecutionResources {
    priority: ThreadPriority, //TBD
    stack_size: StackSize,
    //Behavior?
}

impl Description {
    pub fn new(id: String, channel_tx: Option<TX>) -> Self {
        Self { id, channel_tx }
    }
    pub fn get_id(&self) -> String {
        self.id.clone()
    }
    pub fn get_address(&self) -> Option<TX> {
        self.channel_tx.clone()
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

pub(crate) trait PrivateGenericEntity {
    fn set_aid(&mut self, aid: Description);
    fn set_thread_id(&mut self, thread_id: ID);
    fn set_receiver(&mut self, rx: RX);
}

pub trait GenericEntity {
    //this trait will define top level gets and actions like recv and send msg
    fn get_aid(&self) -> Option<Description>;
    fn get_nickname(&self) -> String;
    fn get_resources(&self) -> ExecutionResources;
    fn get_thread_id(&self) -> Option<ID>;
    //MESSAGING GOES HERE
}


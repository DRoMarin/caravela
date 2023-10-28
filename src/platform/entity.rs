pub mod dispatcher;
use crate::platform::{StackSize, ThreadPriority, ID, TX};

#[derive(Clone)]
pub struct Description {
    id: String,
    channel_tx: Option<TX>,
}

pub(crate) struct ExecutionResources {
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

pub trait GenericEntity {
    //getters
    fn get_aid(&self) -> Option<Description>;
    fn get_name(&self) -> String;
    fn get_priority(&self) -> ThreadPriority;
    fn get_stack_size(&self) -> usize;
    fn get_thread_id(&self) -> Option<ID>;
}
//TRAIT FOR MESSAGING. NEEDED FOR ALL ENTITIES. Implementation is not universal across entities
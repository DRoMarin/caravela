pub mod behavior;
pub mod organization;

use std::{
    collections::HashMap,
    sync::{atomic::Ordering, mpsc::channel},
    thread::{current, Thread},
};

use crate::platform::{
    entity::{messaging::Message, Description, Entity, ExecutionResources},
    ControlBlock, Directory, StackSize, ThreadPriority, MAX_SUBSCRIBERS, RX,
};

//use behavior::Behavior;

pub struct AgentHub {
    nickname: String,
    pub aid: Description,
    pub resources: ExecutionResources,
    rx: RX,
    pub msg: Message,
    pub directory: Directory,
    pub(crate) control_block: Option<ControlBlock>,
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
        let (tx, rx) = channel::<Message>();
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
            //membership,
        }
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

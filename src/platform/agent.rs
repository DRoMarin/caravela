pub mod behavior;
pub mod organization;

use std::collections::HashMap;

use crate::platform::{
    entity::{messaging::Message, Description, ExecutionResources, GenericEntity},
    Directory, ID, MAX_SUBSCRIBERS, RX,
};

use super::entity::PrivateGenericEntity;

pub struct AgentHub {
    nickname: String,
    pub aid: Option<Description>,
    pub resources: ExecutionResources,
    receiver: Option<RX>,
    pub msg: Message,
    thread_id: Option<ID>,
    pub directory: Directory,
    //membership: Option<Membership<'a>>,
}

pub struct Agent<T> {
    pub hub: AgentHub,
    pub data: T,
    //pub membership,
}

impl AgentHub {
    pub(crate) fn new(nickname: String, resources: ExecutionResources) -> Self {
        let msg = Message::new();
        let directory: Directory = HashMap::with_capacity(MAX_SUBSCRIBERS);
        Self {
            nickname,
            aid: None,
            resources,
            receiver: None,
            msg,
            thread_id: None,
            directory,
            //membership,
        }
    }
}
impl PrivateGenericEntity for AgentHub {
    //Setters
    fn set_aid(&mut self, aid: Description) {
        self.aid = Some(aid);
    }
    fn set_thread_id(&mut self, thread_id: ID) {
        self.thread_id = Some(thread_id);
    }
    fn set_receiver(&mut self, rx: RX) {
        self.receiver = Some(rx);
    }
}

impl GenericEntity for AgentHub {
    //Getters
    fn get_aid(&self) -> Option<Description> {
        self.aid.clone()
    }
    fn get_nickname(&self) -> String {
        self.nickname.clone()
    }
    fn get_resources(&self) -> ExecutionResources {
        self.resources.clone()
    }
    fn get_thread_id(&self) -> Option<ID> {
        self.thread_id.clone()
    }
    //Messaging needed
}

impl<T> Agent<T> {}

mod private_task_control {
    //THIS SHOULD PROVIDE
    use super::Agent;
    pub trait TaskControl {
        //TBD
        fn bar(&self) {}
    }
    impl<T> TaskControl for Agent<T> {}
}

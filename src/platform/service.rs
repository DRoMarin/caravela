use std::{
    process::id,
    sync::{Arc, Mutex},
};

use crate::platform::{
    entity::{
        messaging::Message, Description, ExecutionResources, GenericEntity, PrivateGenericEntity,
    },
    Directory, Platform, ID, RX,
};

pub mod ams;
pub mod mts;

struct ServiceHub {
    nickname: String,
    pub aid: Option<Description>,
    pub resources: ExecutionResources,
    receiver: Option<RX>,
    pub msg: Message,
    thread_id: Option<ID>,
    pub directory: Arc<Mutex<Directory>>,
}

impl ServiceHub {
    pub(crate) fn new(
        nickname: String,
        resources: ExecutionResources,
        directory: Arc<Mutex<Directory>>,
    ) -> Self {
        let msg = Message::new();
        Self {
            nickname,
            aid: None,
            resources,
            receiver: None,
            msg,
            thread_id: None,
            directory,
        }
    }
}

impl PrivateGenericEntity for ServiceHub {
    fn set_aid(&mut self, aid: Description) {
        self.aid = Some(aid);
    }
    fn set_receiver(&mut self, rx: RX) {
        self.receiver = Some(rx)
    }
    fn set_thread_id(&mut self, thread_id: crate::platform::ID) {
        self.thread_id = Some(thread_id);
    }
}
impl GenericEntity for ServiceHub {
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
}

pub(crate) trait Service {
    fn new(platform: &Platform) -> Self;
    fn service_function(conditions: &impl UserConditions);
}

pub trait UserConditions {
    fn registration_condition(&self) -> bool {
        true
    }
    fn deregistration_condition(&self) -> bool {
        true
    }
    fn suspension_condition(&self) -> bool {
        true
    }
    fn resumption_condition(&self) -> bool {
        true
    }
    fn termination_condition(&self) -> bool {
        true
    }
    fn reset_condition(&self) -> bool {
        true
    }
}

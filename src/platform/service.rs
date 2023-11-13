use std::{sync::mpsc::sync_channel, thread::Thread};

use crate::platform::{
    entity::{messaging::Message, Description, Entity, ExecutionResources},
    ErrorCode, Platform, RX,
};

use super::entity::messaging::MessageType;

pub mod ams;

struct ServiceHub {
    nickname: String,
    pub aid: Description,
    pub resources: ExecutionResources,
    receiver: RX,
    pub msg: Message,
}

impl ServiceHub {
    pub(crate) fn new(
        nickname: String,
        resources: ExecutionResources,
        thread: Thread,
        hap: &str,
    ) -> Self {
        let (tx, rx) = sync_channel::<Message>(1);
        let name = nickname.clone() + "@" + hap;
        let aid = Description::new(name, tx, thread);
        let msg = Message::new();
        Self {
            nickname,
            aid,
            resources,
            receiver: rx,
            msg,
        }
    }
}

impl Entity for ServiceHub {
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
        //TBD: PLACEHOLDER
        ErrorCode::NoError
    }
    fn receive(&mut self) -> MessageType {
        //TBD: PLACEHOLDER
        self.msg.get_type().clone().unwrap()
    }
}

pub(crate) trait Service {
    type Conditions;
    fn new(hap: &Platform, thread: Thread, conditions: Self::Conditions) -> Self;
    fn register_agent(&mut self, nickname: &str, description: Description) -> ErrorCode;
    fn deregister_agent(&mut self, nickname: &str) -> ErrorCode;
    fn search_agent(&self, nickname: &str) -> ErrorCode; // TBD
    fn service_function(conditions: Self::Conditions);
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

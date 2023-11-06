use std::{
    sync::{mpsc::channel, Arc, Mutex},
    thread::Thread,
};

use crate::platform::{
    entity::{messaging::Message, Description, Entity, ExecutionResources},
    Platform, RX,
};

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
        let (tx, rx) = channel::<Message>();
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
}

pub(crate) trait Service<T: UserConditions> {
    fn new(hap: &Platform, thread: Thread, conditions: T) -> Self;
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

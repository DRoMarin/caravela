use std::sync::{
    mpsc::{sync_channel, TrySendError},
    Arc, RwLock,
};

use crate::platform::{
    entity::{messaging::Message, Description, Entity, ExecutionResources},
    Directory, ErrorCode, Platform, RX,
};

use super::entity::messaging::MessageType;

pub mod ams;

pub struct DefaultConditions;
pub(crate) struct ServiceHub {
    nickname: String,
    pub aid: Description,
    pub resources: ExecutionResources,
    rx: RX,
    pub msg: Message,
    directory: Arc<RwLock<Directory>>,
}

impl ServiceHub {
    pub(crate) fn new(
        nickname: String,
        resources: ExecutionResources,
        hap: &str,
        directory: Arc<RwLock<Directory>>,
    ) -> Self {
        let (tx, rx) = sync_channel::<Message>(1);
        let name = nickname.clone() + "@" + hap;
        let aid = Description::new(name, tx, None);
        let msg = Message::new();
        Self {
            nickname,
            aid,
            resources,
            rx,
            msg,
            directory,
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
        let receiver = match self.directory.read().unwrap().get(agent) {
            Some(x) => x.clone(),
            None => return ErrorCode::NotRegistered,
        };
        self.msg.set_sender(self.aid.clone());
        let address = receiver.get_address().clone();
        self.msg.set_receiver(receiver);
        let result = address.try_send(self.msg.clone());
        let error_code = match result {
            Ok(_) => ErrorCode::NoError,
            Err(error) => match error {
                TrySendError::Full(_) => ErrorCode::Timeout,
                TrySendError::Disconnected(_) => ErrorCode::NotRegistered, //LIST MAY BE OUTDATED
            },
        };
        error_code
    }
    fn receive(&mut self) -> MessageType {
        let result = self.rx.recv();
        let msg_type = match result {
            Ok(received_msg) => {
                self.msg = received_msg;
                self.msg.get_type().clone().unwrap()
            }
            Err(_) => MessageType::NoResponse,
        }; //could handle Err incase of disconnection
        msg_type
    }
}

pub(crate) trait Service {
    type Conditions;
    fn new(hap: &Platform, conditions: Self::Conditions) -> Self;
    fn register_agent(&mut self, nickname: &str, description: Description) -> ErrorCode;
    fn deregister_agent(&mut self, nickname: &str) -> ErrorCode;
    fn search_agent(&self, nickname: &str) -> ErrorCode; // TBD
    fn service_function(&mut self);
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

impl UserConditions for DefaultConditions {}

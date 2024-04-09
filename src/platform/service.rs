use crate::platform::{
    entity::{messaging::Message, Description, ExecutionResources},
    ErrorCode, Platform, RX,
};
use std::sync::mpsc::sync_channel;

use super::entity::messaging::{MessageType, RequestType};

pub mod ams;

pub struct DefaultConditions;

pub(crate) struct ServiceHub {
    nickname: String,
    hap: String,
    pub aid: Description,
    pub resources: ExecutionResources,
    rx: RX,
    pub msg: Message,
    //deck: Arc<RwLock<Deck>>,
}

impl ServiceHub {
    pub(crate) fn new(
        nickname: String,
        resources: ExecutionResources,
        platform: &str,
    ) -> Self {
        let (tx, rx) = sync_channel::<Message>(1);
        let hap = platform.to_string();
        let name = nickname.clone() + "@" + &hap;
        let aid = Description::new(name, tx, None);
        let msg = Message::new();
        Self {
            nickname,
            hap,
            aid,
            resources,
            rx,
            msg,
            //deck,
        }
    }
}

pub(crate) trait Service {
    type Conditions;
    fn new(hap: &Platform, conditions: Self::Conditions) -> Self;
    fn register_agent(&mut self, nickname: &str) -> ErrorCode;
    fn deregister_agent(&mut self, nickname: &str) -> ErrorCode;
    fn search_agent(&self, nickname: &str) -> ErrorCode; // TBD
    fn service_function(&mut self);
    fn service_request_reply_type(&mut self, request_type: RequestType, error: ErrorCode);
    fn error_to_msgtype(err: ErrorCode) -> MessageType {
        match err {
            ErrorCode::Found => MessageType::Inform,
            _ => MessageType::Failure,
        }
    }
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

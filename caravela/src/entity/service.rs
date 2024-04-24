pub(crate) mod ams;

use std::sync::{Arc, RwLock};

use crate::{deck::Deck, entity::messaging::RequestType, ErrorCode};

#[derive(Debug)]
pub struct DefaultConditions;

pub(crate) trait Service {
    type Conditions;
    fn new(hap: String, deck: Arc<RwLock<Deck>>, conditions: Self::Conditions) -> Self;
    fn register_agent(&mut self, name: &str) -> Result<(), ErrorCode>;
    fn deregister_agent(&mut self, name: &str) -> Result<(), ErrorCode>;
    fn search_agent(&self, name: &str) -> Result<(), ErrorCode>;
    fn service_function(&mut self);
    fn service_req_reply_type(&mut self, request_type: RequestType, result: Result<(), ErrorCode>);
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

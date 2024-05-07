pub(crate) mod ams;

use std::sync::{Arc, RwLock};

use crate::{deck::Deck, entity::messaging::RequestType, Description, ErrorCode, RX};

#[derive(Debug)]
pub struct DefaultConditions;

pub(crate) trait Service {
    type Conditions;
    fn new(
        aid: Description,
        //resources: ExecutionResources,
        rx: RX,
        deck: Arc<RwLock<Deck>>,
        conditions: Self::Conditions,
    ) -> Self;
    fn init(&mut self);
    fn register_agent(&mut self, aid: &Description) -> Result<(), ErrorCode>;
    fn deregister_agent(&mut self, aid: &Description) -> Result<(), ErrorCode>;
    fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode>;
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

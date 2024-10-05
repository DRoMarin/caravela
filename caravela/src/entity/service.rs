pub(crate) mod ams;

use crate::{
    entity::{messaging::RequestType, Description},
    ErrorCode, RX,
};

#[derive(Debug)]
pub(crate) struct DefaultConditions;

pub(crate) trait Service {
    type Conditions;
    fn new(rx: RX, conditions: Self::Conditions) -> Self;
    fn init(&mut self);
    fn register_agent(&mut self, aid: &Description) -> Result<(), ErrorCode>;
    fn deregister_agent(&mut self, aid: &Description) -> Result<(), ErrorCode>;
    fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode>;
    fn service_function(&mut self);
    fn service_req_reply_type(&mut self, request_type: RequestType, result: Result<(), ErrorCode>);
}

/// This trait defines a set of boolean functions whose purpose is to specify
///  under which conditions an entity like the AMS can provide each service:
///  - Registration
///  - Deregistration
///  - Suspension
///  - Resumption
///  - Termination
///  - Reset
pub trait UserConditions {
    /// Whether or not it is possible to register an agent to the internal directory;
    /// a White Pages (WP) directory in the case of the AMS.
    fn registration_condition(&self) -> bool {
        true
    }

    /// Whether or not it is possible to deregister an agent from the internal directory;
    /// a White Pages (WP) directory in the case of the AMS.
    fn deregistration_condition(&self) -> bool {
        true
    }

    /// Whether or not it is possible to suspend an agent. This is only doable by the AMS.
    fn suspension_condition(&self) -> bool {
        true
    }

    /// Whether or not it is possible to resume an agent. This is only doable by the AMS.
    fn resumption_condition(&self) -> bool {
        true
    }

    /// Whether or not it is possible to suspend an agent. This is only doable by the AMS.
    fn termination_condition(&self) -> bool {
        true
    }

    /// Whether or not it is possible to restart an agent, which means to terminate and relaunch the agent.
    ///  This is only doable by the AMS.
    fn reset_condition(&self) -> bool {
        true
    }
}

impl UserConditions for DefaultConditions {}

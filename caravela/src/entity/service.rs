pub(crate) mod ams;

use crate::{
    entity::{messaging::MessageType, Content, Description},
    ErrorCode,
};

use super::messaging::Message;

#[derive(Debug)]
pub(crate) struct DefaultConditions;

pub(crate) trait Service {
    fn name(&self) -> String;
    fn init(&mut self);
    fn register_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode>;
    fn deregister_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode>;
    fn search_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode>;
    fn service_function(&mut self);
    fn request_reply(
        &self,
        receiver: Description,
        message_type: MessageType,
        content: Content,
    ) -> Result<(), ErrorCode>;
}

/// This trait defines a set of boolean functions whose purpose is to specify
///  under which conditions any service entity should provide its services:
///  - Registration
///  - Deregistration
pub trait ServiceConditions {
    /// Whether or not it is possible to search an agent in the internal directory;
    /// a White Pages (WP) directory in the case of the AMS.
    fn search_condition(&self) -> bool {
        true
    }
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
}

/// This trait defines a set of boolean functions whose purpose is to specify
///  under which conditions the AMS should provide its specific services:
///  - Suspension
///  - Resumption
///  - Termination
///  - Reset
///
/// This trait is a subtrait of [`ServiceConditions`]
pub trait AmsConditions: ServiceConditions {
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

impl ServiceConditions for DefaultConditions {}
impl AmsConditions for DefaultConditions {}

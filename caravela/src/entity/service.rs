pub(crate) mod ams;

use crate::{
    entity::{
        messaging::{Content, MessageType},
        Description,
    },
    ErrorCode,
};

#[derive(Debug)]
pub(crate) struct DefaultConditions;

pub(crate) trait Service {
    fn name(&self) -> String;
    fn init(&mut self);
    fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode>;
    //fn modify_agent(&self, aid: &Description, modify: &ModifyAgent) -> Result<(), ErrorCode>;
    fn modify_agent(&self, aid: &Description, modifier: &str) -> Result<(), ErrorCode>;
    fn register_agent(&self, aid: &Description) -> Result<(), ErrorCode>;
    fn deregister_agent(&self, aid: &Description) -> Result<(), ErrorCode>;
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
/// 
///  - Search
///  - Modification
///  - Registration
///  - Deregistration
/// 
/// Whenever the internal directory is referenced, it corresponds to 
///  the White Pages directory for requests aimed at the AMS.
pub trait ServiceConditions {
    /// Whether or not it is possible to search an agent in the internal directory;
    fn search_condition(&self) -> bool {
        true
    }
    /// Whether or not it is possible to modify an agent registered in the internal directory;
    fn modification_condition(&self) -> bool {
        true
    }
    /// Whether or not it is possible to register an agent to the internal directory;
    fn registration_condition(&self) -> bool {
        true
    }

    /// Whether or not it is possible to deregister an agent from the internal directory;
    fn deregistration_condition(&self) -> bool {
        true
    }
}

/// This trait defines a set of boolean functions whose purpose is to specify
///  under which conditions the AMS should provide its specific services:
/// 
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

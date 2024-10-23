pub(crate) mod ams;
pub(crate) mod organization;

pub use ams::AmsConditions;
use crate::{
    entity::{
        messaging::{Content, MessageType},
        Description,
    },
    ErrorCode,
};

pub(crate) trait Service {
    fn name(&self) -> String;
    fn init(&mut self);
    fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode>;
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

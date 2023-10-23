use self::base::AgentInfoDescription;
use crate::platform::AgentPrio;

pub mod base;
pub mod behavior;
pub mod organization;
pub trait GenericAgent {
    //getters
    fn get_aid(&self) -> Option<&AgentInfoDescription>;
    fn get_name(&self) -> &str;
    fn get_priority(&self) -> AgentPrio;
    fn get_stack_size(&self) -> usize;
    //fn get_directory(&self) -> &T;
    //setters
    //fn set_aid(&mut self, aid: AgentInfoDescription);
    //fn set_platform(&mut self, platform: &Platform);
}

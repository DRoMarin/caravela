use self::base::{Agent, AgentInfoDescription};
use crate::platform::AgentPrio;

pub mod base;
pub mod behavior;
pub mod organization;
pub trait GenericAgent {
    //getters
    fn get_aid(&self) -> Option<&AgentInfoDescription>;
    fn get_name(&self) -> String;
    fn get_priority(&self) -> AgentPrio;
    fn get_stack_size(&self) -> usize;
    //fn get_directory(&self) -> &T;
    //setters
    //fn set_aid(&mut self, aid: AgentInfoDescription);
    //fn set_platform(&mut self, platform: &Platform);
}

pub struct AgentWrapper<'a, T> {
    agent: &'a Agent<'a>,
    data: T,
}
mod privateTaskControl { //THIS SHOULD PROVIDE 
    use super::AgentWrapper;
    pub trait TaskControl {
        //TBD
        fn bar(&self){}
    }

    impl<'a, T> TaskControl for AgentWrapper<'a, T> {}
}

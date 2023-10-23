use crate::platform::agent::base::AgentInfoDescription;

pub mod ams;
pub mod mts;

pub(crate) trait Service<T> {
    fn get_aid(&self) -> &AgentInfoDescription;
    fn get_name(&self) -> &str;
}
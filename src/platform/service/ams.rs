use crate::platform::agent::base::{Agent, ExecutionResources, Info};
use crate::platform::{AgentPrio, Directory, Generic, AID};
use std::collections::HashMap;

pub(crate) struct WhitePages<'a>(HashMap<AID, &'a Agent>);
pub (crate) struct AMSAgent<'a> {
    info: Info,
    resources: ExecutionResources,
    pub(crate) directory: WhitePages<'a>,
}

impl<'a> Generic<WhitePages<'a>> for AMSAgent<'a> {
    fn get_aid(&self) -> Option<AID> {
        self.info.aid
    }
    fn get_name(&self) -> &str {
        &self.info.name
    }
    fn get_platform(&self) -> Option<AID> {
        self.info.platform
    }
    fn get_priority(&self) -> AgentPrio {
        self.resources.priority
    }
    fn get_stack_size(&self) -> usize {
        self.resources.stack_size
    }
    fn get_directory(&self) -> &WhitePages<'a> {
        &self.directory
    }
    fn set_aid(&mut self, aid: AID) {
        self.info.aid = Some(aid);
    }
    fn set_platform(&mut self, platform_aid: AID) {
        self.info.platform = Some(platform_aid);
    }
}

impl<'a> Directory<HashMap<AID, &'a Agent>, (AID, &'a Agent)> for WhitePages<'a> {
    fn add_element(&mut self, element: (AID, &'a Agent)) {
        self.0.insert(element.0, element.1);
    }
    fn remove_element(&mut self, element: (AID, &'a Agent)) {
        self.0.remove(&element.0);
    }
    fn get_directory(&self) -> &HashMap<AID, &'a Agent> {
        &self.0
    }
    fn clear_directory(&mut self) {
        self.0.clear();
    }
    fn refresh_directory(&mut self) {

    }
}
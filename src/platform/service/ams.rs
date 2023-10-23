use crate::platform::agent::base::{Agent, AgentInfoDescription};
use crate::platform::agent::GenericAgent;
use crate::platform::message::Message;
use crate::platform::service::{Directory, Service, UserConditions};
use crate::platform::{ErrorCode, Platform, MAX_SUBSCRIBERS};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

pub(crate) struct WhitePages<'a>(HashMap<AgentInfoDescription, &'a Agent<'a>>);
pub(crate) struct AMSService<'a> {
    //TODO: this can become a generic ServiceAgent<S> struct:
    //become ServiceAgent<AMS> or ServiceAgent<DF>
    name: String,
    aid: AgentInfoDescription,
    pub directory: WhitePages<'a>,
}

impl<'a> Service<WhitePages<'a>> for AMSService<'a> {
    fn get_aid(&self) -> &AgentInfoDescription {
        &self.aid
    }
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl<'a>
    Directory<HashMap<AgentInfoDescription, &'a Agent<'a>>, AgentInfoDescription, &'a Agent<'a>>
    for WhitePages<'a>
{
    fn add_element(&mut self, key: AgentInfoDescription, value: &'a Agent<'a>) {
        self.0.insert(key, value);
    }
    fn get_element(&self, element: AgentInfoDescription) -> Option<&'a Agent<'a>> {
        self.0.get(&element).copied()
    }
    fn remove_element(&mut self, element: AgentInfoDescription) {
        self.0.remove(&element);
    }
    fn get_directory(&self) -> &HashMap<AgentInfoDescription, &'a Agent<'a>> {
        &self.0
    }
    fn clear_directory(&mut self) {
        self.0.clear();
    }
    fn refresh_directory(&mut self) {}
}

impl<'a> AMSService<'a> {
    pub(crate) fn new(platform: &'a Platform) -> Self {
        let name = "AMS".to_string();
        let id = name.clone() + "@" + &platform.name.clone();
        let aid = AgentInfoDescription::new(id);
        //let resources = ExecutionResources::new(MAX_PRIORITY, DEFAULT_STACK);
        let directory = WhitePages(HashMap::with_capacity(MAX_SUBSCRIBERS));
        Self {
            name,
            aid,
            directory,
        }
    }
    pub(crate) fn register_agent(
        &mut self,
        agent: &'a mut Agent<'a>,
        cond: &impl UserConditions,
    ) -> ErrorCode {
        if cond.registration_condition() {
            let aid: AgentInfoDescription = AgentInfoDescription::new(agent.get_name()); //FIX ID NAMING
            if !self.directory.0.contains_key(&aid) {
                agent.set_aid(aid.clone());
                let agent_entry = &*agent;
                self.directory.add_element(aid.clone(), agent_entry);
                return ErrorCode::NoError;
            } else {
                return ErrorCode::Duplicated;
            }
        }
        ErrorCode::Invalid
    }
    pub(crate) fn deregister_agent(&mut self, agent_aid: &AgentInfoDescription) {}
    pub(crate) fn kill_agent(&mut self, agent_aid: &AgentInfoDescription) {}
    pub(crate) fn suspend_agent(&self, agent_aid: &AgentInfoDescription) {}
    pub(crate) fn resume_agent(&mut self, agent_aid: &AgentInfoDescription) {}
    pub(crate) fn restart_agent(&mut self, agent_aid: &AgentInfoDescription) {}
}

//fn format_id(name:,platform)
//IDEA
//let a: &std::sync::mpsc::Receiver<Message> = &self.resources.channel.as_ref().unwrap().1;
//let msg = a.recv();

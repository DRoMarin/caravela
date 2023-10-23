use crate::platform::agent::base::{Agent, AgentInfoDescription};
use crate::platform::message::Message;
use crate::platform::service::{Directory, Service};
use crate::platform::{ErrorCode, Platform, UserConditions, ID, MAX_SUBSCRIBERS};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::thread::current;

pub(crate) struct WhitePages<'a>(HashMap<AgentInfoDescription<'a>, &'a Agent<'a>>);
pub(crate) struct AMSService<'a> {
    //TODO: this can become a generic ServiceAgent<S> struct:
    //become ServiceAgent<AMS> or ServiceAgent<DF>
    name: String,
    aid: AgentInfoDescription<'a>,
    pub(crate) directory: WhitePages<'a>,
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
    Directory<
        HashMap<AgentInfoDescription<'a>, &'a Agent<'a>>,
        AgentInfoDescription<'a>,
        &'a Agent<'a>,
    > for WhitePages<'a>
{
    fn add_element(&mut self, key: AgentInfoDescription<'a>, value: &'a Agent<'a>) {
        self.0.insert(key, value);
    }
    fn get_element(&self, element: AgentInfoDescription<'a>) -> Option<&'a Agent<'a>> {
        self.0.get(&element).copied()
    }
    fn remove_element(&mut self, element: AgentInfoDescription<'a>) {
        self.0.remove(&element);
    }
    fn get_directory(&self) -> &HashMap<AgentInfoDescription<'a>, &'a Agent<'a>> {
        &self.0
    }
    fn clear_directory(&mut self) {
        self.0.clear();
    }
    fn refresh_directory(&mut self) {}
}

impl<'a> AMSService<'a> {
    pub(crate) fn new(platform: &'a Platform) -> Self {
        let aid = AgentInfoDescription::new(current().id(), platform);
        //let resources = ExecutionResources::new(MAX_PRIORITY, DEFAULT_STACK);
        let directory = WhitePages(HashMap::with_capacity(MAX_SUBSCRIBERS));
        Self {
            name: "AMS".to_string(),
            aid,
            directory,
        }
    }
    pub(crate) fn register_agent(&self, agent: Agent<'a>) -> ErrorCode {
        /*match agent_aid {
            None => ErrorCode::HandleNone,
            Some(aid) => if !self.directory.0.contains_key(aid){
                self.directory.add_element(key, value)
            }
        }*/
        ErrorCode::NoError
    }
    pub(crate) fn deregister_agent(&self, agent_aid: &AgentInfoDescription) {}
    pub(crate) fn kill_agent(&self, agent_aid: &AgentInfoDescription) {}
    pub(crate) fn suspend_agent(&self, agent_aid: &AgentInfoDescription) {}
    pub(crate) fn resume_agent(&self, agent_aid: &AgentInfoDescription) {}
    pub(crate) fn restart_agent(&self, agent_aid: &AgentInfoDescription) {}
}

//IDEA
//let a: &std::sync::mpsc::Receiver<Message> = &self.resources.channel.as_ref().unwrap().1;
//let msg = a.recv();

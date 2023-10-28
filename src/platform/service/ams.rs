use crate::platform::{
    agent::AgentInfo,
    entity::{Description, ExecutionResources, GenericEntity},
    message::Message,
    service::{Directory, Service, UserConditions},
    {ErrorCode, Platform, DEFAULT_STACK, MAX_PRIORITY, MAX_SUBSCRIBERS, RX, TX},
};
use std::collections::HashMap;
use std::sync::mpsc::channel;

type DirectoryEntry = HashMap<String, TX>;

pub(crate) struct WhitePages(DirectoryEntry);
pub(crate) struct AMSService {
    //TODO: this can become a generic ServiceAgent<S> struct:
    //become ServiceAgent<AMS> or ServiceAgent<DF>
    name: String,
    aid: Description,
    resources: ExecutionResources,
    directory: WhitePages,
    rx_channel: RX, // will become message dispatcher struct
}

impl Service<WhitePages> for AMSService {
    fn get_aid(&self) -> Description {
        self.aid.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl Directory<DirectoryEntry, String, TX> for WhitePages {
    fn add_element(&mut self, key: String, value: TX) {
        self.0.insert(key, value);
    }
    fn get_element(&self, element: String) -> Option<TX> {
        self.0.get(&element).cloned()
    }
    fn remove_element(&mut self, element: String) {
        self.0.remove(&element);
    }
    fn get_directory(&self) -> DirectoryEntry {
        self.0.clone()
    }
    fn clear_directory(&mut self) {
        self.0.clear();
    }
    fn refresh_directory(&mut self) {}
}

// impl GenericEntity for AMSService {}

impl AMSService {
    pub(crate) fn new(platform: &Platform) -> Self {
        let name = "AMS".to_string();
        let id = name.clone() + "@" + &platform.name.clone();
        let (tx, rx) = channel::<Message>();
        let aid = Description::new(id, Some(tx));
        let resources = ExecutionResources::new(MAX_PRIORITY, DEFAULT_STACK);
        let directory = WhitePages(HashMap::with_capacity(MAX_SUBSCRIBERS));
        Self {
            name,
            aid,
            resources,
            directory,
            rx_channel: rx,
        }
    }

    pub(crate) fn register_agent(
        &mut self,
        agent: &mut AgentInfo,
        cond: &impl UserConditions,
    ) -> ErrorCode {
        if !cond.registration_condition() {
            return ErrorCode::Invalid;
        }
        let id = agent.get_name();
        if self.directory.0.contains_key(&id) {
            return ErrorCode::Duplicated;
        }
        let (tx, rx) = channel::<Message>();
        let aid: Description = Description::new(id.clone(), Some(tx.clone())); //FIX ID NAMING
        agent.set_aid(aid);
        agent.set_rx(rx);
        self.directory.add_element(id, tx);
        return ErrorCode::NoError;
    }
    pub(crate) fn deregister_agent(&mut self, agent_aid: &Description) {}
    pub(crate) fn kill_agent(&mut self, agent_aid: &Description) {}
    pub(crate) fn suspend_agent(&self, agent_aid: &Description) {}
    pub(crate) fn resume_agent(&mut self, agent_aid: &Description) {}
    pub(crate) fn restart_agent(&mut self, agent_aid: &Description) {}
}

//fn format_id(name:,platform)
//IDEA
//let a: &std::sync::mpsc::Receiver<Message> = &self.resources.channel.as_ref().unwrap().1;
//let msg = a.recv();

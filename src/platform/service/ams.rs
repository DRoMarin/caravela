use crate::platform::{
    agent::AgentHub,
    entity::{
        messaging::Message, Description, ExecutionResources, GenericEntity, PrivateGenericEntity,
    },
    service::{Service, ServiceHub, UserConditions},
    {ErrorCode, Platform, DEFAULT_STACK, MAX_PRIORITY, MAX_SUBSCRIBERS},
};
use std::sync::mpsc::channel;

//SERVICE NEEDS TO CHANGE TO FIT DISPATCHER STRUCT > GET RID OF MONO STRUCT, CREATE TYPES (CONTACTS, YP,WP,ETC)

struct AMS {
    //become Service<AMS> or Service<DF>
    service_hub: ServiceHub,
    platform_name: String,
}

impl Service for AMS {
    fn new(platform: &Platform) -> Self {
        let nickname = "AMS".to_string();
        let platform_name = platform.name.clone();
        let id = nickname.clone() + "@" + &platform_name;
        let (tx, rx) = channel::<Message>();
        let aid = Description::new(id, Some(tx));
        let resources = ExecutionResources::new(MAX_PRIORITY, DEFAULT_STACK);
        let mut service_hub =
            ServiceHub::new(nickname.clone(), resources, platform.white_pages.clone());
        service_hub.set_aid(aid);
        service_hub.set_receiver(rx);
        Self {
            service_hub,
            platform_name,
        }
    }
    fn service_function(conditions: &impl UserConditions) {}
}
impl AMS {
    pub(crate) fn register_agent(
        &mut self,
        agent_hub: &mut AgentHub,
        cond: &impl UserConditions,
    ) -> ErrorCode {
        if !cond.registration_condition() {
            return ErrorCode::Invalid;
        }
        let mut white_pages = self.service_hub.directory.lock().unwrap();
        if white_pages.capacity().eq(&MAX_SUBSCRIBERS) {
            return ErrorCode::ListFull;
        }
        let nickname = agent_hub.get_nickname();
        if white_pages.contains_key(&nickname) {
            return ErrorCode::Duplicated;
        }
        let (tx, rx) = channel::<Message>();
        let id = nickname.clone() + "@" + &self.platform_name;
        let aid = Description::new(id.clone(), Some(tx.clone()));
        agent_hub.set_aid(aid);
        agent_hub.set_receiver(rx);
        white_pages.insert(nickname, (id, tx));
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

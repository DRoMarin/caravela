pub mod behavior;
pub mod organization;

use crate::platform::{
    entity::{dispatcher::Message, Description, ExecutionResources, GenericEntity},
    ThreadPriority, ID, RX,
};
use std::sync::mpsc::Receiver;

use super::entity::dispatcher::{MessageDispatcher, self};

pub struct AgentInfo {
    nickname: String,
    aid: Option<Description>,
    //platform: Option<String>,
    resources: ExecutionResources,
    dispatcher: Option<MessageDispatcher>,//Option<Receiver<Message>>, // will become message dispatcher struct
    thread_id: Option<ID>,
    //membership: Option<Membership<'a>>,
}

pub struct Agent<T> {
    agent: AgentInfo,
    data: T,
}

/*
pub(crate) struct Membership<'a> {
    org: Option<&'a Organization<'a>>,
    affiliation: Option<OrgAffiliation>,
    role: Option<OrgRole>,
}
*/
/*trait OrgMember {
    //getters
    fn get_org(&self) -> &Organization;
    fn get_affiliation(&self) -> Option<OrgAffiliation>;
    fn get_role(&self) -> Option<OrgRole>;
    //setters
    fn set_affiliation(&mut self, affiliation: OrgAffiliation);
    fn set_role(&mut self, role: OrgRole);
}*/

impl AgentInfo {
    pub fn new(nickname: String, priority: i32, stack_size: usize) -> Self {
        let aid = None;
        let resources = ExecutionResources::new(priority, stack_size);
        let dispatcher = None; 
        //let membership = None;

        Self {
            nickname,
            aid,
            //platform: None,
            resources,
            dispatcher,
            thread_id: None, //contact_list: ContactList(Vec::<AID>::with_capacity(MAX_SUBSCRIBERS)),
                             //membership,
        }
    }
    pub(crate) fn set_thread_id(&mut self, thread_id: ID) {
        self.thread_id = Some(thread_id);
    }
    pub(crate) fn set_aid(&mut self, aid: Description) {
        self.aid = Some(aid);
    }
    pub(crate) fn set_dispatcher(&mut self, rx: RX) {
        self.dispatcher = Some(MessageDispatcher::new(rx));
    }
}

impl GenericEntity for AgentInfo {
    fn get_aid(&self) -> Option<Description> {
        self.aid.clone()
    }
    fn get_name(&self) -> String {
        self.nickname.clone()
    }
    fn get_priority(&self) -> ThreadPriority {
        self.resources.get_priority()
    }
    fn get_stack_size(&self) -> usize {
        self.resources.get_stack_size()
    }
    fn get_thread_id(&self) -> Option<ID> {
        self.thread_id.clone()
    }
}

mod private_task_control {
    //THIS SHOULD PROVIDE
    use super::Agent;
    pub trait TaskControl {
        //TBD
        fn bar(&self) {}
    }
    impl<T> TaskControl for Agent<T> {}
}

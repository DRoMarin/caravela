use crate::platform::agent::organization::{OrgAffiliation, OrgRole, Organization};
use crate::platform::message::Message;
use crate::platform::{AgentPrio, Platform, StackSize, ID};
use crate::platform::agent::GenericAgent;
use std::sync::mpsc::{Receiver, Sender};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct AgentInfoDescription<'a> {
    id: ID,
    platform: &'a Platform,
}

pub(crate) struct ExecutionResources {
    priority: AgentPrio, //TBD
    stack_size: StackSize,
    //Behavior?
}

pub(crate) struct Membership<'a> {
    org: Option<&'a Organization<'a>>,
    affiliation: Option<OrgAffiliation>,
    role: Option<OrgRole>,
}

pub struct Agent<'a> {
    name: String,
    aid: Option<AgentInfoDescription<'a>>,
    membership: Option<Membership<'a>>,
    resources: ExecutionResources,
    channel: Option<(Sender<Message>,Receiver<Message>)>,
}

/*trait OrgMember {
    //getters
    fn get_org(&self) -> &Organization;
    fn get_affiliation(&self) -> Option<OrgAffiliation>;
    fn get_role(&self) -> Option<OrgRole>;
    //setters
    fn set_affiliation(&mut self, affiliation: OrgAffiliation);
    fn set_role(&mut self, role: OrgRole);
}*/

impl<'a> AgentInfoDescription<'a> {
    pub fn new(id: ID, platform: &'a Platform) -> Self {
        Self { id, platform }
    }
    pub fn get_id(&self) -> ID {
        self.id
    }
    pub fn get_platform(&self) -> &Platform {
        self.platform
    }
}

impl ExecutionResources {
    pub fn new(priority: AgentPrio, stack_size: StackSize) -> Self {
        Self {
            priority,
            stack_size,
        }
    }
    pub fn get_priority(&self) -> AgentPrio {
        self.priority
    }
    pub fn get_stack_size(&self) -> usize {
        self.stack_size
    }
}

impl<'a> Agent<'a> {
    pub fn new(name: String, priority: i32, stack_size: usize) -> Self {
        let aid = None;
        let resources = ExecutionResources::new(priority, stack_size);
        let membership = None;

        Self {
            name,
            aid,
            membership,
            resources,
            channel: None,
            //contact_list: ContactList(Vec::<AID>::with_capacity(MAX_SUBSCRIBERS)),
        }
    }
}

impl<'a> GenericAgent for Agent<'a> {
    fn get_aid(&self) -> Option<&AgentInfoDescription<'a>> {
        self.aid.as_ref()
    }
    /*fn get_mailbox_tx(&self) -> Option<TX> {
        if self.info.channel.is_none() {
            return None;
        }

        return Some(self.info.channel.as_ref().unwrap().0.clone());
    }*/
    fn get_name(&self) -> &str {
        &self.name
    }
    fn get_priority(&self) -> AgentPrio {
        self.resources.get_priority()
    }
    fn get_stack_size(&self) -> usize {
        self.resources.get_stack_size()
    }
}

/*
impl<'a> OrgMember for Membership<'a> {
    fn get_org(&self) -> Option<&Organization> {
        if self.membership.is_none() {
            return None
        }
        self.membership.unwrap().org
    }
    fn get_affiliation(&self) -> Option<OrgAffiliation> {
        self.membership.affiliation
    }
    fn get_role(&self) -> Option<OrgRole> {
        self.membership.role
    }
    fn set_affiliation(&mut self, affiliation: OrgAffiliation) {
        self.membership.affiliation = Some(affiliation);
    }
    fn set_role(&mut self, role: OrgRole) {
        self.membership.role = Some(role);
    }
}
*/
/*
impl Directory<Vec<AID>, ID, Option<ID>> for ContactList {
    fn add_element(&mut self, element: AID) {
        self.0.push(element);
    }
    fn get_element(&self, element: AID) -> Option<AID> {
        let index = self.0.iter().position(|x| *x == element);
        if index.is_none() {
            return None;
        } else {
            return Some(self.0[index.unwrap()]);
        }
    }
    fn remove_element(&mut self, element: AID) {
        let index = self.0.iter().position(|x| *x == element);
        if index.is_some() {
            self.0.remove(index.unwrap());
        }
    }
    fn get_directory(&self) -> &Vec<AID> {
        &self.0
    }
    fn clear_directory(&mut self) {
        self.0.clear();
    }
    fn refresh_directory(&mut self) {
        todo!()
    }
}
*/

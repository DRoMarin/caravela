use super::MAX_SUBSCRIBERS;
use crate::platform::agent::organization::{OrgAffiliation, OrgRole, Organization};
use crate::platform::{AgentPrio, Directory, Generic, StackSize, AID};

pub(crate) struct Info {
    pub aid: Option<AID>,
    pub name: String,
    pub platform: Option<AID>,
}

pub(crate) struct Membership {
    org: Option<Organization>,
    affiliation: Option<OrgAffiliation>,
    role: Option<OrgRole>,
}

pub(crate) struct ExecutionResources {
    pub priority: AgentPrio, //TBD
    pub stack_size: StackSize,
    //Behavior:
}
pub struct ContactList(Vec<AID>);

pub struct Agent {
    info: Info,
    membership: Membership,
    resources: ExecutionResources,
    pub contact_list: ContactList,
}

pub fn new(name: String, priority: i32, stack_size: usize) -> Agent {
    let info = Info {
        aid: None,
        name,
        platform: None,
    };
    let membership = Membership {
        org: None,
        affiliation: Some(OrgAffiliation::NonMember),
        role: Some(OrgRole::NoRole),
    };
    let resources = ExecutionResources {
        priority,
        stack_size,
    };
    Agent {
        info,
        membership,
        resources,
        contact_list: ContactList(Vec::<AID>::with_capacity(MAX_SUBSCRIBERS)),
    }
}

trait OrgMember {
    //getters
    fn get_org(&self) -> Option<&Organization>;
    fn get_affiliation(&self) -> Option<OrgAffiliation>;
    fn get_role(&self) -> Option<OrgRole>;  
    //setters
    fn set_affiliation(&mut self, affiliation: OrgAffiliation);
    fn set_role(&mut self, role: OrgRole);
}

impl Generic<ContactList> for Agent {
    fn get_aid(&self) -> Option<AID> {
        self.info.aid
    }
    /*fn get_mailbox_tx(&self) -> Option<TX> {
        if self.info.channel.is_none() {
            return None;
        }

        return Some(self.info.channel.as_ref().unwrap().0.clone());
    }*/
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
    fn get_directory(&self) -> &ContactList {
        &self.contact_list
    }
    fn set_aid(&mut self, aid: AID) {
        self.info.aid = Some(aid);
    }
    fn set_platform(&mut self, platform_aid: AID) {
        self.info.platform = Some(platform_aid);
    }
}

impl OrgMember for Agent {
    fn get_org(&self) -> Option<&Organization> {
        self.membership.org.as_ref()
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

impl Directory<Vec<AID>, AID> for ContactList {
    fn add_element(&mut self, element: AID) {
        self.0.push(element);
    }
    fn remove_element(&mut self, element: AID) {
        let index = self
        .0
        .iter()
        .position(|x| *x == element);
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
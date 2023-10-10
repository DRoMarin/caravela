use crate::platform::agents::organization::{OrgAffiliation, OrgRole,Organization};
use super::{AgentPrio, StackSize, AID, MAX_SUBSCRIBERS};

struct Info {
    aid: Option<AID>,
    name: String,
    platform: Option<AID>,
    org: Option<Organization>,
    affiliation: Option<OrgAffiliation>,
    role: Option<OrgRole>,
}

struct ExecutionResources {
    priority: AgentPrio, //TBD
    stack_size: StackSize,
    //Behavior:
}

pub struct Agent {
    info: Info,
    resources: ExecutionResources,
    contact_list: Vec<AID>,
}
/*pub trait Agent_Constructor {
    fn new(name: String, priority: i32, stack_size: usize) -> Agent;
}
*/
pub(in crate::platform) trait Generic {
    //GET
    fn get_aid(&self) -> Option<AID>;
    fn get_name(&self) -> &str;
    fn get_platform(&self) -> Option<AID>;
    fn get_org(&self) -> Option<&Organization>;
    fn get_affiliation(&self) -> Option<OrgAffiliation>;
    fn get_role(&self) -> Option<OrgRole>;
    fn get_priority(&self) -> AgentPrio;
    fn get_stack_size(&self) -> usize;
    fn get_contact_list(&self) -> &Vec<AID>;
    //SET
    fn set_aid(&mut self, aid: AID);
    fn set_platform(&mut self, platform_aid: AID);
    fn set_affiliation(&mut self, affiliation: OrgAffiliation);
    fn set_role(&mut self, role: OrgRole);
    fn add_contact(&mut self, contact: AID);
    fn remove_contact(&mut self, contact: AID);
    fn clear_contact_list(&mut self);
    fn refresh_contact_list(&mut self);
}

pub fn new(name: String, priority: i32, stack_size: usize) -> Agent {
    let info = Info {
        aid: None,
        name,
        platform: None,
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
        resources,
        contact_list: Vec::<AID>::with_capacity(MAX_SUBSCRIBERS),
    }
}

impl Generic for Agent {
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
    fn get_org(&self) -> Option<&Organization> {
        self.info.org.as_ref()
    }
    fn get_affiliation(&self) -> Option<OrgAffiliation> {
        self.info.affiliation
    }
    fn get_role(&self) -> Option<OrgRole> {
        self.info.role
    }
    fn get_priority(&self) -> AgentPrio {
        self.resources.priority
    }
    fn get_stack_size(&self) -> usize {
        self.resources.stack_size
    }
    fn get_contact_list(&self) -> &Vec<AID> {
        &self.contact_list
    }

    fn set_aid(&mut self, aid: AID) {
        self.info.aid = Some(aid);
    }
    fn set_platform(&mut self, platform_aid: AID) {
        self.info.platform = Some(platform_aid);
    }
    fn set_affiliation(&mut self, affiliation: OrgAffiliation) {
        self.info.affiliation = Some(affiliation);
    }
    fn set_role(&mut self, role: OrgRole) {
        self.info.role = Some(role);
    }
    fn add_contact(&mut self, contact: AID) {
        self.contact_list.push(contact);
    }
    fn remove_contact(&mut self, contact: AID) {
        let index = self
            .contact_list
            .iter()
            .position(|x| *x == contact)
            .unwrap();
        self.contact_list.remove(index);
    }
    fn clear_contact_list(&mut self) {
        self.contact_list.clear();
    }
    fn refresh_contact_list(&mut self) {
        //TBD
    }
}

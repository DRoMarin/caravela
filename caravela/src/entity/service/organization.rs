use crate::{
    entity::{Description, Hub},
    ErrorCode, MAX_SUBSCRIBERS,
};
use std::collections::HashSet;

use super::Service;

#[derive(Debug)]
pub enum OrgAffiliation {
    Owner,
    Admin,
    Member,
    NonMember,
}

#[derive(Debug)]
pub enum OrgRole {
    Moderator,
    Participant,
    Visitor,
    NoRole,
}

#[derive(Debug, Default)]
pub enum OrgType {
    Hierarchy,
    #[default]
    Team,
}

#[derive(Debug, Clone)]
struct OrgRecord {
    members: HashSet<Description>,
    banned: HashSet<Description>,
    owner: Option<Description>,
    admin: Option<Description>,
    moderator: Option<Description>,
}

#[derive(Debug)]
struct Organization {
    org_type: OrgType,
    record: OrgRecord,
    hub: Hub,
}

fn new(org_type: OrgType, hub: Hub) -> Organization {
    let record = OrgRecord {
        members: HashSet::<Description>::with_capacity(MAX_SUBSCRIBERS),
        banned: HashSet::<Description>::with_capacity(MAX_SUBSCRIBERS),
        owner: None,
        admin: None,
        moderator: None,
    };
    Organization {
        org_type,
        record,
        hub,
    }
}

impl OrgRecord {
    fn set_moderator(&mut self, aid: Description) -> Option<Description> {
        self.moderator.replace(aid)
    }
    fn set_owner(&mut self, aid: Description) -> Option<Description> {
        self.moderator.replace(aid)
    }
    fn add_member(&mut self, aid: Description) -> Result<(), ErrorCode> {
        self.members
            .insert(aid)
            .then_some(())
            .ok_or(ErrorCode::Duplicated)
    }
    fn remove_member(&mut self, aid: Description) -> Result<(), ErrorCode> {
        self.members
            .remove(&aid)
            .then_some(())
            .ok_or(ErrorCode::NotFound)
    }
}
impl Service for Organization {
    fn new(rx: crate::RX, deck: DeckAccess, conditions: Self::Conditions) -> Self {}
    fn init(&mut self) {}
    fn register_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {}
    fn deregister_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {}
    fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode> {}
    fn service_function(&mut self) {}
    fn service_req_reply_type(
        &mut self,
        request_type: crate::messaging::RequestType,
        result: Result<(), ErrorCode>,
    ) {
    }

    type Conditions;
}
/*
impl Organization {
    fn add_member(target: AID) -> Result<(), ErrorCode> {}
    fn invite_member(target: AID) -> MessageType {}
    fn kick_member(target: AID) -> Result<(), ErrorCode> {}
    fn ban_member(target: AID) -> Result<(), ErrorCode> {}

    fn change_owner(target: AID) -> Result<(), ErrorCode> {}

    fn set_admin(target: AID) -> Result<(), ErrorCode> {}
    fn set_moderator(target: AID) -> Result<(), ErrorCode> {}
    fn set_participant(target: AID) -> Result<(), ErrorCode> {}
    fn set_visitor(target: AID) -> Result<(), ErrorCode> {}

    fn lift_ban(target: AID) -> Result<(), ErrorCode> {}
    fn clear_ban_list() -> Result<(), ErrorCode> {}

    fn get_size(&self) -> usize {
        self.info.members.len()
    }
    fn get_description(&self) -> &OrgRecord {
        &self.info
    }
    fn get_org_type(&self) -> OrgType {
        self.org_type
    }

    fn is_member(&self, target: &Description) -> bool {
        self.info.members.contains(target)
    }
    fn is_banned(&self, target: &Description) -> bool {
        self.info.banned.contains(target)
    }
}
*/

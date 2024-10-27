use crate::{
    entity::{Description, Hub},
    ErrorCode, MAX_SUBSCRIBERS,
};
use std::collections::HashSet;

use super::{Service, ServiceConditions};

trait OrgConditions: ServiceConditions {}

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
pub(crate) struct OrgRecord {
    members: HashSet<Description>,
    banned: HashSet<Description>,
    owner: Option<Description>,
    admin: Option<Description>,
    moderator: Option<Description>,
}

#[derive(Debug)]
struct Organization<T: OrgConditions> {
    nickname: &'static str,
    hap: &'static str,
    org_type: OrgType,
    record: OrgRecord,
    hub: Hub,
    conditions: T,
}

impl<T: OrgConditions> Organization<T> {
    fn new(
        nickname: &'static str,
        hap: &'static str,
        org_type: OrgType,
        hub: Hub,
        conditions: T,
    ) -> Organization<T> {
        let record = OrgRecord {
            members: HashSet::<Description>::with_capacity(MAX_SUBSCRIBERS),
            banned: HashSet::<Description>::with_capacity(MAX_SUBSCRIBERS),
            owner: None,
            admin: None,
            moderator: None,
        };
        Organization {
            nickname,
            hap,
            org_type,
            record,
            hub,
            conditions,
        }
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
impl<T: OrgConditions> Service for Organization<T> {
    fn name(&self) -> String {
        format!("{}@{}", self.nickname, self.hap)
    }
    fn init(&mut self) {
        caravela_status!("{}: Started!", self.name())
    }
    fn register_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        todo!()
    }
    fn deregister_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        todo!()
    }
    fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        todo!()
    }
    fn service_function(&mut self) {}

    fn modify_agent(&self, aid: &Description, modifier: &str) -> Result<(), ErrorCode> {
        todo!()
    }

    fn request_reply(
        &self,
        receiver: Description,
        message_type: crate::messaging::MessageType,
        content: crate::messaging::Content,
    ) -> Result<(), ErrorCode> {
        todo!()
    }
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

use crate::platform::agent::base::AgentInfoDescription;
use crate::platform::message::MessageType;
use crate::platform::ErrorCode;

use crate::platform::MAX_SUBSCRIBERS;

#[derive(Clone, Copy)]
pub enum OrgAffiliation {
    Owner,
    Admin,
    Member,
    NonMember,
}

#[derive(Clone, Copy)]
pub enum OrgRole {
    Moderator,
    Participant,
    Visitor,
    NoRole,
}

#[derive(Clone, Copy)]
pub enum OrgType {
    Hierarchy,
    Team,
}

#[derive(Clone)]
pub struct OrgInfo<'a> {
    pub org_type: OrgType,
    pub members: Vec<&'a AgentInfoDescription>,
    pub banned: Vec<&'a AgentInfoDescription>,
    pub owner: Option<&'a AgentInfoDescription>,
    pub admin: Option<&'a AgentInfoDescription>,
    pub moderator: Option<&'a AgentInfoDescription>,
}

#[derive(Clone)]
pub struct Organization<'a> {
    info: OrgInfo<'a>,
}

pub fn new<'a>(org_type: OrgType) -> Organization<'a> {
    let info = OrgInfo {
        org_type,
        members: Vec::<&'a AgentInfoDescription>::with_capacity(MAX_SUBSCRIBERS),
        banned: Vec::<&'a AgentInfoDescription>::with_capacity(MAX_SUBSCRIBERS),
        owner: None,
        admin: None,
        moderator: None,
    };
    Organization { info }
}

impl<'a> Organization<'a> {
    /*fn open(&self) -> ErrorCode {
        if current().id() ==  {
            return ErrorCode::Invalid;
        }
        else if self.info.owner == None {
            return ErrorCode::NoError;
        }
        else {
            return ErrorCode::Invalid;
        }
    }*/
    //AFTER WHITE PAGES DONE
    /*
        fn close() -> ErrorCode {}

        fn add_member(target: AID) -> ErrorCode {}
        fn invite_member(target: AID) -> MessageType {}
        fn kick_member(target: AID) -> ErrorCode {}
        fn ban_member(target: AID) -> ErrorCode {}

        fn change_owner(target: AID) -> ErrorCode {}

        fn set_admin(target: AID) -> ErrorCode {}
        fn set_moderator(target: AID) -> ErrorCode {}
        fn set_participant(target: AID) -> ErrorCode {}
        fn set_visitor(target: AID) -> ErrorCode {}

        fn lift_ban(target: AID) -> ErrorCode {}
        fn clear_ban_list() -> ErrorCode {}

        fn get_size(&self) -> usize {
            self.info.members.len()
        }
        fn get_info(&self) -> OrgInfo {
            self.info
        }
        fn get_org_type(&self) -> OrgType {
            self.info.org_type
        }

        fn is_member(&self, target:&AgentInfoDescription<'_> ) -> ErrorCode {
            if self.info.members.contains(&target) {
                return ErrorCode::Found;
            }
            else {
                return ErrorCode::NotFound;
            }
        }

        fn is_banned(&self, target:&AgentInfoDescription<'_> ) -> ErrorCode {
            if self.info.banned.contains(&target) {
                return ErrorCode::Found;
            }
            else {
                return ErrorCode::NotFound;
            }
        }
    */
}

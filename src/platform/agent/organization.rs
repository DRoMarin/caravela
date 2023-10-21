use std::thread::current;
use crate::platform::agent::AID;
use crate::platform::{ErrorCode};
use crate::platform::message::MessageType;

use super::MAX_SUBSCRIBERS;

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
pub struct OrgInfo {
    pub org_type: OrgType,
    pub members: Vec<AID>,
    pub banned: Vec<AID>,
    pub owner: Option<AID>,
    pub admin: Option<AID>,
    pub moderator: Option<AID>,
}

#[derive(Clone)]
pub struct Organization {
    info: OrgInfo,
}

pub fn new(org_type: OrgType) -> Organization {
    let info = OrgInfo {
        org_type,
        members: Vec::<AID>::with_capacity(MAX_SUBSCRIBERS),
        banned: Vec::<AID>::with_capacity(MAX_SUBSCRIBERS),
        owner: None,
        admin: None,
        moderator: None,
    };
    Organization { info }
}

impl Organization {
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
    }*/ //AFTER WHITE PAGES DONE
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
*/

    fn is_member(&self, target: AID) -> ErrorCode {
        if self.info.members.contains(&target) {
            return ErrorCode::Found;    
        }
        else {
            return ErrorCode::NotFound;
        }
    }  

    fn is_banned(&self, target: AID) -> ErrorCode {    
        if self.info.banned.contains(&target) {
            return ErrorCode::Found;    
        }
        else {
            return ErrorCode::NotFound;
        }
    }

}

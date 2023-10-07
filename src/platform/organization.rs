pub enum OrgAffiliation {
    Owner,
    Admin,
    Member,
    NonMember,
}

pub enum OrgRole {
    Moderator,
    Participant,
    Visitor,
    NoRole,
}

pub enum OrgType {
    Hierarchy,
    Team,
}

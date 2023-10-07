use crate::platform::{message, organization};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::ThreadId;

struct StackSize(usize);

struct Info {
    aid: Option<ThreadId>,
    mailbox_rx: Option<Receiver<message::Message>>,
    mailbox_tx: Option<Sender<message::Message>>,
    name: Option<String>,
    priority: Option<i32>, //TBD
    platform: Option<std::thread::ThreadId>,
    //org:
    affiliation: Option<organization::OrgAffiliation>,
    role: Option<organization::OrgRole>,
}

pub struct Agent {
    info: Info,
    stack_size: StackSize,
}

pub(in crate::platform) trait Generic {
    fn get_aid(&self) -> Option<ThreadId>;
}

fn new(name: String, prio: i32, size: usize) -> Agent {
    let info = Info {
        aid: None,
        mailbox_rx: None,
        mailbox_tx: None,
        name: Some(name),
        priority: Some(prio),
        platform: None,
        affiliation: Some(organization::OrgAffiliation::NonMember),
        role: Some(organization::OrgRole::NoRole),
    };
    let stack_size = StackSize(size);

    Agent {
        info: info,
        stack_size: stack_size,
    }
}

impl Generic for Agent {
    fn get_aid(&self) -> Option<ThreadId> {
        self.info.aid
    }
}

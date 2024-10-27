use crate::{
    agent::AgentState,
    entity::{agent::ControlBlockArc, Description},
    service::organization::OrgRecord,
    ErrorCode, MAX_SUBSCRIBERS,
};
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
    thread::{JoinHandle, Thread, ThreadId},
};
use thread_priority::ThreadPriority;

pub(crate) type AgentDirectory = HashMap<Description, AgentEntry>;
pub(crate) type OrgDirectory = HashMap<Description, OrgEntry>;

#[derive(Debug)]
pub(crate) struct AmsEntry {
    aid: Description,
    #[allow(dead_code)]
    join_handle: JoinHandle<()>,
}

impl AmsEntry {
    pub(crate) fn aid(&self) -> &Description {
        &self.aid
    }
}

#[derive(Debug)]
pub(crate) struct OrgEntry {
    join_handle: JoinHandle<()>,
    org_record: OrgRecord,
}

impl OrgEntry {}

#[derive(Debug)]
pub(crate) struct AgentEntry {
    pub(crate) join_handle: JoinHandle<()>,
    priority: ThreadPriority,
    control_block: ControlBlockArc,
}

impl AgentEntry {
    pub(crate) fn control_block(&self) -> ControlBlockArc {
        self.control_block.clone()
    }

    pub(crate) fn priority(&self) -> ThreadPriority {
        self.priority
    }

    pub(crate) fn thread(&self) -> Thread {
        self.join_handle.thread().clone()
    }
}

#[derive(Debug)]
pub(crate) struct DeckAccess(RwLock<Deck>);

impl DeckAccess {
    pub(crate) fn new() -> DeckAccess {
        DeckAccess(RwLock::new(Deck::new()))
    }
    pub(crate) fn write(&self) -> RwLockWriteGuard<Deck> {
        self.0
            .write()
            .expect("Deck is poisoned - Lost agent records")
    }
    pub(crate) fn read(&self) -> RwLockReadGuard<Deck> {
        self.0
            .read()
            .expect("Deck is poisoned - Lost agent records")
    }
}

#[derive(Debug)]
pub struct Deck {
    ams_entry: Option<AmsEntry>,
    agent_directory: AgentDirectory,
    #[cfg(feature = "organizations")]
    org_directory: OrgDirectory,
}

impl Deck {
    pub(crate) fn new() -> Self {
        let ams_entry = None;
        let agent_directory = AgentDirectory::with_capacity(MAX_SUBSCRIBERS);
        #[cfg(feature = "organizations")]
        let org_directory = OrgDirectory::with_capacity(MAX_SUBSCRIBERS);
        Self {
            ams_entry,
            agent_directory,
            #[cfg(feature = "organizations")]
            org_directory,
        }
    }

    pub(crate) fn ams_aid(&self) -> &Description {
        self.ams_entry
            .as_ref()
            .expect("Platform has not been booted yet")
            .aid()
    }

    pub(crate) fn add_ams(&mut self, aid: Description, join_handle: JoinHandle<()>) {
        self.ams_entry = Some(AmsEntry { aid, join_handle });
    }

    pub(crate) fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        self.agent_directory
            .contains_key(aid)
            .then_some(())
            .ok_or(ErrorCode::NotRegistered)
    }

    pub(crate) fn get_agent(&self, aid: &Description) -> Result<&AgentEntry, ErrorCode> {
        self.agent_directory
            .get(aid)
            .ok_or(ErrorCode::NotRegistered)
    }

    pub(crate) fn add_agent(
        &mut self,
        aid: Description,
        join_handle: JoinHandle<()>,
        priority: ThreadPriority,
        control_block: ControlBlockArc,
    ) -> Result<(), ErrorCode> {
        if self.search_agent(&aid).is_err() {
            let agent_entry = AgentEntry {
                join_handle,
                priority,
                control_block,
            };
            self.agent_directory.insert(aid.clone(), agent_entry);
            Ok(())
        } else {
            Err(ErrorCode::Duplicated)
        }
    }

    pub(crate) fn modify_agent(
        &self,
        aid: &Description,
        state: AgentState,
    ) -> Result<(), ErrorCode> {
        let entry = self.get_agent(aid)?;
        match state {
            AgentState::Active => {
                entry.control_block().active()?;
                Ok(entry.join_handle.thread().unpark())
            }
            AgentState::Suspended => entry.control_block().suspend(),
            AgentState::Terminated => {
                entry.control_block().quit()
                //join?
            }
            _ => Err(ErrorCode::InvalidStateChange(
                entry.control_block().agent_state(),
                state,
            )),
        }
    }

    pub(crate) fn remove_agent(&mut self, aid: &Description) -> Result<AgentEntry, ErrorCode> {
        self.agent_directory
            .remove(aid)
            .ok_or(ErrorCode::NotRegistered)
    }

    pub(crate) fn get_aid_from_name(&self, name: &str) -> Result<Description, ErrorCode> {
        self.agent_directory
            .keys()
            .find(|aid| aid.name() == *name)
            .cloned()
            .ok_or(ErrorCode::NotFound)
    }

    pub(crate) fn get_aid_from_thread(&self, id: ThreadId) -> Result<Description, ErrorCode> {
        //.find(|aid| aid.id().is_some_and(|x| x == id))
        self.agent_directory
            .keys()
            .find(|aid| aid.id().eq(&Some(id)))
            .cloned()
            .ok_or(ErrorCode::NotFound)
    }

    #[cfg(feature = "organizations")]
    pub(crate) fn get_org_from_name(&self, name: &str) -> Result<Description, ErrorCode> {
        self.org_directory
            .keys()
            .find(|aid| aid.name() == *name)
            .cloned()
            .ok_or(ErrorCode::NotFound)
    }
}

static DECK: OnceLock<DeckAccess> = OnceLock::new();

pub(crate) fn get_deck() -> Option<&'static DeckAccess> {
    DECK.get()
}
pub(crate) fn deck() -> &'static DeckAccess {
    DECK.get_or_init(DeckAccess::new)
}

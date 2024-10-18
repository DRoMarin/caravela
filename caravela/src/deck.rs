use crate::{
    agent::AgentState,
    entity::{agent::ControlBlockArc, Description},
    ErrorCode, MAX_SUBSCRIBERS,
};
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
    thread::{JoinHandle, Thread, ThreadId},
};
use thread_priority::ThreadPriority;

pub(crate) type AgentDirectory = HashMap<Description, AgentEntry>;
//pub(crate) type AmsDirectory = HashMap<Description, AmsEntry>;

#[derive(Debug)]
pub(crate) struct AmsEntry {
    aid: Description,
    join_handle: JoinHandle<()>,
}

impl AmsEntry {
    pub(crate) fn aid(&self) -> &Description {
        &self.aid
    }
    //pub(crate) fn address(&self) -> &Tx {
    //    &self.address
    //}
}

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
    //ams_directory: AmsDirectory,
    agent_directory: AgentDirectory,
}

impl Deck {
    pub(crate) fn new() -> Self {
        let ams_entry = None;
        let agent_directory = AgentDirectory::with_capacity(MAX_SUBSCRIBERS);
        //let ams_directory = AmsDirectory::with_capacity(MAX_SUBSCRIBERS);
        Self {
            ams_entry,
            //ams_directory,
            agent_directory,
        }
    }
    /*pub(crate) fn get_ams_address_for_hap(&self, name: &str) -> Result<Description, ErrorCode> {
        self.ams_directory
            .keys()
            .find(|x| *x.hap() == *name)
            .cloned()
            .ok_or(ErrorCode::NotFound)
    }*/

    pub(crate) fn ams_aid(&self) -> &Description {
        self.ams_entry
            .as_ref()
            .expect("Platform has not been booted yet")
            .aid()
    }

    pub(crate) fn add_ams(&mut self, aid: Description, join_handle: JoinHandle<()>) {
        /*self.ams_directory.insert(
            aid,
            AmsEntry {
                //address,
                join_handle,
            },
        );*/

        self.ams_entry = Some(AmsEntry {
            aid,
            //address,
            join_handle,
        });
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
        //address: Tx,
    ) -> Result<(), ErrorCode> {
        if self.search_agent(&aid).is_err() {
            let agent_entry = AgentEntry {
                join_handle,
                priority,
                control_block,
                //address,
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
            .find(|x| x.name() == *name)
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
}

static DECK: OnceLock<DeckAccess> = OnceLock::new();

pub(crate) fn deck() -> &'static DeckAccess {
    DECK.get_or_init(DeckAccess::new)
}

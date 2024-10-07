use crate::{
    entity::{agent::ControlBlockArc, messaging::Message, Description},
    ErrorCode, MAX_SUBSCRIBERS,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::Ordering,
        mpsc::{SendError, TrySendError},
        OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
    thread::{JoinHandle, Thread, ThreadId},
};
use thread_priority::ThreadPriority;

pub(crate) type AgentDirectory = HashMap<Description, AgentEntry>;

#[derive(Debug)]
pub(crate) enum SyncType {
    Blocking,
    #[allow(dead_code)]
    NonBlocking, //USE?
}

#[derive(Debug)]
pub(crate) enum SendResult {
    Blocking(Result<(), SendError<Message>>),
    NonBlocking(Result<(), TrySendError<Message>>),
}

#[derive(Debug)]
pub(crate) enum TcbField {
    Suspend,
    Quit,
}

#[derive(Debug, Default)]
pub(crate) struct AmsEntry {
    aid: Description,
    join_handle: Option<JoinHandle<()>>,
}

impl AmsEntry {
    pub(crate) fn join(&mut self) -> Result<(), ErrorCode> {
        self.join_handle
            .take()
            .unwrap()
            .join()
            .map_err(|_| ErrorCode::AgentPanic)
    }

    pub(crate) fn aid(&self) -> &Description {
        &self.aid
    }
}

#[derive(Debug)]
pub(crate) struct AgentEntry {
    join_handle: Option<JoinHandle<()>>,
    priority: Option<ThreadPriority>,
    control_block: ControlBlockArc,
}

impl AgentEntry {
    pub(crate) fn join(&mut self) -> Result<(), ErrorCode> {
        self.join_handle
            .take()
            .unwrap()
            .join()
            .map_err(|_| ErrorCode::AgentPanic)
    }

    pub(crate) fn control_block(&self) -> ControlBlockArc {
        self.control_block.clone()
    }

    pub(crate) fn priority(&self) -> Option<ThreadPriority> {
        self.priority
    }

    pub(crate) fn thread(&self) -> Option<Thread> {
        self.join_handle.as_ref().map(|x| x.thread().to_owned())
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
            .unwrap_or_else(|_| panic!("Deck is poisoned - Lost agent records"))
    }
    pub(crate) fn read(&self) -> RwLockReadGuard<Deck> {
        self.0
            .read()
            .unwrap_or_else(|_| panic!("Deck is poisoned - Lost agent records"))
    }
}

#[derive(Debug)]
pub struct Deck {
    ams_entry: AmsEntry,
    agent_directory: AgentDirectory,
}

impl Deck {
    pub(crate) fn new() -> Self {
        let ams_entry = AmsEntry::default();
        let agent_directory = AgentDirectory::with_capacity(MAX_SUBSCRIBERS);
        Self {
            ams_entry,
            agent_directory,
        }
    }

    pub(crate) fn ams_aid(&self) -> &Description {
        self.ams_entry.aid()
    }

    pub(crate) fn ams_join(&mut self) -> Result<(), ErrorCode> {
        self.ams_entry.join()
    }

    pub(crate) fn assign_ams(&mut self, ams_aid: Description, join_handle: JoinHandle<()>) {
        self.ams_entry = AmsEntry {
            aid: ams_aid,
            join_handle: Some(join_handle),
        };
    }

    pub(crate) fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        self.agent_directory
            .contains_key(aid)
            .then_some(())
            .ok_or(ErrorCode::NotRegistered)
    }

    pub(crate) fn add_agent(
        &mut self,
        aid: Description,
        join_handle: Option<JoinHandle<()>>,
        priority: Option<ThreadPriority>,
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

    pub(crate) fn get_agent(&self, aid: &Description) -> Result<&AgentEntry, ErrorCode> {
        self.agent_directory
            .get(aid)
            .ok_or(ErrorCode::NotRegistered)
    }

    //pub(crate) fn modify_agent(&self) -> Result<(), ErrorCode> {}

    pub(crate) fn remove_agent(&mut self, aid: &Description) -> Result<AgentEntry, ErrorCode> {
        self.agent_directory
            .remove(aid)
            .ok_or(ErrorCode::NotRegistered)
    }

    pub(crate) fn unpark_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        let entry = self.get_agent(aid)?;
        //TODO: ERROR NEEDS TO BE CHANGED
        entry
            .join_handle
            .as_ref()
            .map(|x| x.thread().unpark())
            .ok_or(ErrorCode::InvalidRequest)
    }

    pub(crate) fn modify_control_block(
        &mut self,
        aid: &Description,
        field: TcbField,
        val: bool,
    ) -> Result<(), ErrorCode> {
        self.search_agent(aid)?;
        let control_block = self.get_agent(aid)?.control_block();
        match field {
            TcbField::Suspend => control_block.suspend().store(val, Ordering::Relaxed),

            TcbField::Quit => control_block.quit().store(val, Ordering::Relaxed),
        };
        Ok(())
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

    pub(crate) fn send_msg(&self, msg: Message, sync: SyncType) -> Result<(), ErrorCode> {
        caravela_messaging!(
            "{}: Sending {} to {}",
            msg.sender(),
            msg.message_type(),
            msg.receiver()
        );
        //check memberships and roles
        let receiver_aid = msg.receiver();
        let address = receiver_aid.address()?;
        let result = match sync {
            SyncType::Blocking => SendResult::Blocking(address.send(msg)),
            SyncType::NonBlocking => SendResult::NonBlocking(address.try_send(msg)),
        };
        match result {
            SendResult::Blocking(Ok(_)) => Ok(()),
            SendResult::NonBlocking(Ok(_)) => Ok(()),
            SendResult::Blocking(Err(SendError(_))) => Err(ErrorCode::Disconnected),
            SendResult::NonBlocking(Err(error)) => match error {
                TrySendError::Disconnected(_) => Err(ErrorCode::Disconnected), //LIST MAY BE OUTDATED
                TrySendError::Full(_) => Err(ErrorCode::ChannelFull),
            },
        }
        //FIX
    }
    /* add service request protocols */
}

static DECK: OnceLock<DeckAccess> = OnceLock::new();

pub(crate) fn deck() -> &'static DeckAccess {
    DECK.get_or_init(DeckAccess::new)
}

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock, RwLockReadGuard},
    thread::{JoinHandle, Thread, ThreadId},
};

use crate::{agent::ControlBlock, Description, ErrorCode};

/*pub(crate) enum EntityEntry {
    Service(JoinHandle<()>),
    Agent(AgentEntityEntry),
}*/

pub(crate) type AidDirectory = HashMap<String, Description>;
pub(crate) type AgentDirectory = HashMap<Description, AgentEntry>;
pub(crate) type ServiceDirectory = HashMap<Description, JoinHandle<()>>;
//pub(crate) type OrganizationDirectory = HashMap<Description, EntityEntry>;

#[derive(Debug, Default)]
pub(crate) struct EntityEnvironment {
    aid_directory: AidDirectory,
    agent_directory: AgentDirectory,
    service_directory: ServiceDirectory,
    //organization_directory;,
}

impl EntityEnvironment {
    pub(crate) fn aid_directory(&self) -> &AidDirectory {
        &self.aid_directory
    }

    pub(crate) fn agent_directory(&self) -> &AgentDirectory {
        &self.agent_directory
    }

    pub(crate) fn insert_agent(
        &mut self,
        aid: Description,
        join_handle: JoinHandle<()>,
        control_block: Arc<ControlBlock>,
    ) {
        self.aid_directory.insert(aid.name(), aid.clone());
        self.agent_directory.insert(
            aid,
            AgentEntry {
                join_handle,
                control_block,
            },
        );
    }

    pub(crate) fn insert_service(&mut self, aid: Description, handle: JoinHandle<()>) {
        self.aid_directory.insert(aid.name(), aid.clone());
        self.service_directory.insert(aid, handle);
    }
}

#[derive(Debug)]
pub(crate) struct AgentEntry {
    join_handle: JoinHandle<()>,
    control_block: Arc<ControlBlock>,
}

impl AgentEntry {
    pub(crate) fn join_handle_ref(&self) -> &JoinHandle<()> {
        &self.join_handle
    }

    pub(crate) fn control_block(&self) -> Arc<ControlBlock> {
        self.control_block.clone()
    }

    pub(crate) fn thread(&self) -> Thread {
        self.join_handle.thread().clone()
    }
}

/*#[derive(Debug)]
pub(crate) struct AgentEntityEntry {
    pub(crate) join_handle: JoinHandle<()>,
    pub(crate) control_block: Arc<ControlBlock>,
}*/

//pub(crate) type PlatformDirectory = HashMap<Description, EntityEntry>;

//pub(crate) static PLATFORM_ENV: OnceLock<RwLock<PlatformDirectory>> = OnceLock::new();
pub(crate) static PLATFORM_ENV: OnceLock<RwLock<EntityEnvironment>> = OnceLock::new();
//pub(crate) static AID_ENV: OnceLock<RwLock<AidDirectory>> = OnceLock::new();

//pub(crate) fn platform_env() -> &'static RwLock<PlatformDirectory> {
//PLATFORM_ENV.get_or_init(|| RwLock::new(PlatformDirectory::new()))
pub(crate) fn platform_env() -> &'static RwLock<EntityEnvironment> {
    PLATFORM_ENV.get_or_init(|| RwLock::new(EntityEnvironment::default()))
}

pub(crate) fn aid_from_name(name: &str) -> Result<Description, ErrorCode> {
    if let Some(env) = PLATFORM_ENV.get() {
        if let Ok(guard) = env.read() {
            if let Some(aid) = guard.aid_directory().get(name) {
                Ok(aid.clone())
            } else {
                Err(ErrorCode::AidHandleNone)
            }
        } else {
            Err(ErrorCode::PoisonedLock)
        }
    } else {
        Err(ErrorCode::UninitEnv)
    }
}

pub(crate) fn aid_from_thread(id: ThreadId) -> Result<Description, ErrorCode> {
    if let Some(env) = PLATFORM_ENV.get() {
        if let Ok(guard) = env.read() {
            if let Some(entry) = guard
                .aid_directory()
                .iter()
                .find(|(_, aid)| aid.id() == Some(id))
            {
                Ok(entry.1.clone())
            } else {
                Err(ErrorCode::AidHandleNone)
            }
        } else {
            Err(ErrorCode::PoisonedLock)
        }
    } else {
        Err(ErrorCode::UninitEnv)
    }
}
pub(crate) fn enviroment_read_lock(
) -> Result<RwLockReadGuard<'static, EntityEnvironment>, ErrorCode> {
    if let Ok(lock) = platform_env().read() {
        Ok(lock)
    } else {
        Err(ErrorCode::PoisonedLock)
    }
}

pub(crate) fn agent_control_block(aid: &Description) -> Result<Arc<ControlBlock>, ErrorCode> {
    if let Some(entry) = enviroment_read_lock()?.agent_directory().get(aid) {
        Ok(entry.control_block())
    } else {
        Err(ErrorCode::NotFound)
    }
}

pub(crate) fn agent_thread_handle(aid: &Description) -> Result<Thread, ErrorCode> {
    if let Some(entry) = enviroment_read_lock()?.agent_directory().get(aid) {
        Ok(entry.thread())
    } else {
        Err(ErrorCode::NotFound)
    }
}

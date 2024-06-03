use crate::{
    deck::Deck,
    entity::{
        agent::{
            behavior::{execute, Behavior},
            Agent, ControlBlock,
        },
        service::{ams::Ams, DefaultConditions, Service, UserConditions},
        Description,
    },
    platform::environment::{agent_thread_handle, aid_from_name, insert_env, EntityType},
    ErrorCode, Message, DEFAULT_STACK,
};
use std::{
    collections::HashMap,
    hint,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::sync_channel,
        Arc, RwLock,
    },
    thread,
};
use thread_priority::{ThreadBuilderExt, ThreadExt, ThreadPriority, ThreadPriorityValue};

//pub mod organization;
pub(crate) mod environment;

type SpinLockMap = HashMap<Description, (Arc<AtomicBool>, ThreadPriority)>;

/// Represents the Host Agent Platform (HAP) and
///  provides the user with methods to incorporate agents into it.
#[derive(Debug)]
pub struct Platform {
    name: &'static str,
    deck: Arc<RwLock<Deck>>,
    spinlock_map: SpinLockMap,
}

impl Platform {
    /// Function that constructs a new [`Platform`] object with the provided name.
    pub fn new(name: &'static str) -> Self {
        let deck = Arc::new(RwLock::new(Deck::new()));
        let spinlock_map = SpinLockMap::new();
        Self {
            name,
            //ams_aid: &AMS_LOCK,
            deck,
            spinlock_map,
        }
    }

    /// Returns the name of the platform.
    pub fn name(&self) -> &'static str {
        self.name
    }
    /// This method starts the Agent Management System (AMS) as [`boot_with_ams_conditions`](Self::boot_with_ams_conditions) also does,
    ///  but with default service conditions.
    pub fn boot(&mut self) -> Result<(), ErrorCode> {
        let default: DefaultConditions = DefaultConditions;
        self.boot_with_ams_conditions(default)
    }

    /// This method starts the Agent Management System (AMS) with specific user given conditions,
    ///  passed as a type that implements [`UserConditions`].
    pub fn boot_with_ams_conditions<T: UserConditions + Send + 'static>(
        &mut self,
        conditions: T,
    ) -> Result<(), ErrorCode> {
        //
        let (tx, rx) = sync_channel::<Message>(1);
        let mut ams_aid = Description::new("AMS", self.name(), tx);
        let mut ams = Ams::<T>::new(rx, self.deck.clone(), conditions);
        let mut ams_aid_task = ams_aid.clone();
        //
        let ams_handle = thread::Builder::new()
            .stack_size(DEFAULT_STACK)
            .spawn_with_priority(ThreadPriority::Max, move |_| {
                ams_aid_task.set_thread(thread::current().id());
                ams.hub.set_aid(ams_aid_task);
                println!("[INFO] {}: Booting AMS", ams.hub.aid());
                ams.service_function();
            });
        if let Ok(handle) = ams_handle {
            if handle.is_finished() {
                return Err(ErrorCode::AmsBoot);
            }
            //Build description and insert in env lock
            ams_aid.set_thread(handle.thread().id());
            insert_env(ams_aid, EntityType::Service(handle))
        } else {
            Err(ErrorCode::AmsBoot)
        }
    }

    /// This method creates agents of the given `T` type that implements [`Behavior`]
    ///  with the specified parameters (nickname, priority, and stack size).
    ///  If successful, it will return a `Ok(aid)` with the [`Description`] of the agent.
    ///  This Agent is not active by default and must be started by [`start`](Self::start)
    pub fn add<T: Behavior + Send + 'static>(
        &mut self,
        nickname: &'static str,
        priority: u8,
        stack_size: usize,
    ) -> Result<Description, ErrorCode> {
        // build agent
        let hap = self.name;
        let (tx, rx) = sync_channel::<Message>(1);
        let mut aid = Description::new(nickname, hap, tx);
        let tcb = Arc::new(ControlBlock::default());
        let deck = self.deck.clone();
        let base_agent = Agent::new(rx, deck, tcb.clone());
        if aid_from_name(&base_agent.aid().name()) != Err(ErrorCode::AidHandleNone) {
            return Err(ErrorCode::Duplicated);
        }
        let agent = T::agent_builder(base_agent);

        // check prio
        if priority == ThreadPriorityValue::MAX {
            return Err(ErrorCode::InvalidPriority(
                "Max priority only allowed for Services",
            ));
        }
        let priority = match ThreadPriority::try_from(priority) {
            Ok(agent_priority) => agent_priority,
            Err(error) => return Err(ErrorCode::InvalidPriority(error)),
        };

        // spawn agent with spinlock
        let spinlock = Arc::new(AtomicBool::new(false));
        let spinlock_platform = spinlock.clone();
        let agent_handle = thread::Builder::new()
            .stack_size(stack_size)
            .spawn_with_priority(ThreadPriority::Min, move |_| {
                while !spinlock.load(Ordering::Acquire) {
                    hint::spin_loop();
                }
                execute(agent);
            });
        // register on env
        if let Ok(handle) = agent_handle {
            /*if handle.is_finished() {
                return Err(ErrorCode::AgentFinish);
            }*/
            //Build description and insert in env lock
            aid.set_thread(handle.thread().id());
            self.spinlock_map
                .insert(aid.clone(), (spinlock_platform, priority));
            insert_env(aid.clone(), EntityType::Agent(handle, tcb))?;
            self.deck.write().unwrap().insert_agent(aid.clone())?;
            Ok(aid)
        } else {
            Err(ErrorCode::AgentPanic)
        }
    }

    /// Transition the agent from the initiated state into the active state, required for it to execute its behavior.
    pub fn start(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        if let Some((spinlock, priority)) = self.spinlock_map.remove(aid) {
            let thread = agent_thread_handle(aid)?;
            if let Err(error) = thread.set_priority(priority) {
                return Err(ErrorCode::AgentStart(error));
            }
            spinlock.store(true, Ordering::Release);
            Ok(())
        } else {
            Err(ErrorCode::NotFound)
        }
    }
    //COULD ADD PLATFORM FUNCTIONS AND CALL THEM FROM AMS AGENT
}

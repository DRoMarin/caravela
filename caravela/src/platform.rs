use crate::{
    agent::AgentBuildParam,
    deck::{Deck, DeckAccess},
    entity::{
        agent::{
            behavior::{execute, AgentBuild, Behavior},
            Agent, ControlBlock, ControlBlockAccess,
        },
        service::{ams::Ams, DefaultConditions, Service, UserConditions},
        Description, Entity,
    },
    ErrorCode, Message, DEFAULT_STACK,
};
use std::{
    sync::{atomic::Ordering, mpsc::sync_channel, Arc, RwLock},
    thread,
};
use thread_priority::{ThreadBuilderExt, ThreadExt, ThreadPriority, ThreadPriorityValue};

//pub mod organization;

/// Represents the Host Agent Platform (HAP) and
///  provides the user with methods to incorporate agents into it.
#[derive(Debug)]
pub struct Platform {
    name: &'static str,
    deck: DeckAccess,
}

impl Platform {
    /// Function that constructs a new [`Platform`] object with the provided name.
    pub fn new(name: &'static str) -> Self {
        let deck = DeckAccess(Arc::new(RwLock::new(Deck::new())));
        Self { name, deck }
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
        caravela_status!("BOOTING AMS");
        let ams_handle = thread::Builder::new()
            .stack_size(DEFAULT_STACK)
            .spawn_with_priority(ThreadPriority::Max, move |_| {
                ams_aid_task.set_thread(thread::current().id());
                ams.set_aid(ams_aid_task);
                ams.service_function();
            });

        if let Ok(join_handle) = ams_handle {
            if join_handle.is_finished() {
                return Err(ErrorCode::AmsBoot);
            }
            //Build description and insert in env lock
            ams_aid.set_thread(join_handle.thread().id());
            self.deck.write()?.assign_ams(ams_aid, join_handle);
            Ok(())
        } else {
            Err(ErrorCode::AmsBoot)
        }
    }

    /// This method creates agents of the given `T` type that implements [`Behavior`]
    ///  with the specified parameters (nickname, priority, and stack size).
    ///  If successful, it will return a `Ok(aid)` with the [`Description`] of the agent.
    ///  This Agent is not active by default and must be started by [`start`](Self::start)
    pub fn add_agent<T: Behavior + AgentBuild + Send + 'static>(
        &mut self,
        nickname: &'static str,
        priority: u8,
        stack_size: usize,
    ) -> Result<Description, ErrorCode> {
        // build agent
        let hap = self.name;
        let (tx, rx) = sync_channel::<Message>(1);
        let mut aid = Description::new(nickname, hap, tx);
        let deck = self.deck.clone();
        let control_block = ControlBlockAccess(Arc::new(ControlBlock::default()));
        let mut base_agent = Agent::new(aid.clone(), rx, deck, control_block.clone());
        base_agent.set_aid(aid.clone());
        if self.deck.read()?.search_agent(&aid).is_ok() {
            return Err(ErrorCode::Duplicated);
        }

        let agent = T::agent_builder(base_agent);

        // check prio
        if priority == ThreadPriorityValue::MAX {
            return Err(ErrorCode::InvalidPriority(
                "Max priority only allowed for Services",
            ));
        }

        let thread_priority =
            ThreadPriority::try_from(priority).map_err(|e| ErrorCode::InvalidPriority(e))?;

        // spawn agent with spinlock

        let agent_handle = thread::Builder::new()
            .stack_size(stack_size)
            .spawn_with_priority(ThreadPriority::Min, move |_| {
                execute(agent);
            });

        // register on env
        let join_handle = agent_handle.map_err(|_| ErrorCode::AgentPanic)?;

        //Build description and insert in env lock
        aid.set_thread(join_handle.thread().id());
        self.deck.write()?.add_agent(
            aid.clone(),
            Some(join_handle),
            Some(thread_priority),
            control_block,
        )?;
        Ok(aid)
    }

    pub fn add_agent_with_param<T: Behavior + AgentBuildParam + Send + 'static>(
        &mut self,
        nickname: &'static str,
        priority: u8,
        stack_size: usize,
        param: T::Parameter,
    ) -> Result<Description, ErrorCode> {
        // build agent
        let hap = self.name;
        let (tx, rx) = sync_channel::<Message>(1);
        let mut aid = Description::new(nickname, hap, tx);
        let deck = self.deck.clone();
        let control_block = ControlBlockAccess(Arc::new(ControlBlock::default()));
        let mut base_agent = Agent::new(aid.clone(), rx, deck, control_block.clone());
        base_agent.set_aid(aid.clone());
        if self.deck.read()?.search_agent(&aid).is_ok() {
            return Err(ErrorCode::Duplicated);
        }

        let agent = T::agent_with_param_builder(base_agent, param);

        // check prio
        if priority == ThreadPriorityValue::MAX {
            return Err(ErrorCode::InvalidPriority(
                "Max priority only allowed for Services",
            ));
        }

        let thread_priority =
            ThreadPriority::try_from(priority).map_err(|e| ErrorCode::InvalidPriority(e))?;

        // spawn agent with spinlock

        let agent_handle = thread::Builder::new()
            .stack_size(stack_size)
            .spawn_with_priority(ThreadPriority::Min, move |_| {
                execute(agent);
            });

        // register on env
        let join_handle = agent_handle.map_err(|_| ErrorCode::AgentPanic)?;

        //Build description and insert in env lock
        aid.set_thread(join_handle.thread().id());
        self.deck.write()?.add_agent(
            aid.clone(),
            Some(join_handle),
            Some(thread_priority),
            control_block,
        )?;
        Ok(aid)
    }

    /// Transition the agent from the initiated state into the active state, required for it to execute its behavior.
    pub fn start(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        let deck = self.deck.read()?;
        let entry = deck.get_agent(aid)?;
        let thread = entry.thread().ok_or(ErrorCode::AidHandleNone)?;
        let priority = entry.priority().ok_or(ErrorCode::InvalidPriority("None"))?;
        if let Err(error) = thread.set_priority(priority) {
            return Err(ErrorCode::AgentStart(error));
        }
        entry
            .control_block()
            .active()
            .store(true, Ordering::Release);
        Ok(())
    }

    //COULD ADD PLATFORM FUNCTIONS AND CALL THEM FROM AMS AGENT
}

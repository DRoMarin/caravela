use crate::{
    deck::{deck, get_deck},
    entity::{
        agent::{
            behavior::{execute, Behavior},
            Agent, AgentBuild, AgentBuildParam, ControlBlock,
        },
        messaging::Message,
        service::{ams::Ams, AmsConditions, DefaultConditions, Service},
        Description,
    },
    ErrorCode, DEFAULT_STACK,
};
use std::{
    sync::{mpsc::sync_channel, Arc},
    thread,
};
use thread_priority::{ThreadBuilderExt, ThreadExt, ThreadPriority, ThreadPriorityValue};

const RESERVED_NAMES: [&str; 1] = ["ams"];

/// Represents the Host Agent Platform (HAP) and
///  provides the user with methods to incorporate agents into it.
#[derive(Debug)]
pub struct Platform {
    name: &'static str,
}

impl Platform {
    /// Function that constructs a new [`Platform`] object with the provided name.
    pub fn new(name: &'static str) -> Result<Self, ErrorCode> {
        if get_deck().is_some() {
            return Err(ErrorCode::PlatformPresent);
        }

        let platform = Self { name };
        platform.boot().map(|_| platform)
    }

    /// Function that constructs a new [`Platform`] object with the provided name and conditions for the AMS.
    pub fn new_with_conditions<T: AmsConditions + Send + 'static>(
        name: &'static str,
        conditions: T,
    ) -> Result<Self, ErrorCode> {
        if get_deck().is_some() {
            return Err(ErrorCode::PlatformPresent);
        }

        let platform = Self { name };
        platform
            .boot_with_ams_conditions(conditions)
            .map(|_| platform)
    }
    /// Returns the name of the platform.
    pub fn name(&self) -> &'static str {
        self.name
    }
    /// This method starts the Agent Management System (AMS) as [`boot_with_ams_conditions`](Self::boot_with_ams_conditions) also does,
    ///  but with default service conditions.
    fn boot(&self) -> Result<(), ErrorCode> {
        let default: DefaultConditions = DefaultConditions;
        self.boot_with_ams_conditions(default)
    }

    /// This method starts the Agent Management System (AMS) with specific user given conditions,
    ///  passed as a type that implements [`AmsConditions`].
    fn boot_with_ams_conditions<T: AmsConditions + Send + 'static>(
        &self,
        conditions: T,
    ) -> Result<(), ErrorCode> {
        let (tx, rx) = sync_channel::<Message>(1);
        let mut ams_aid = Description::new("ams", self.name(), tx);
        let mut ams = Ams::<T>::new(self.name, rx, conditions);

        caravela_status!("BOOTING AMS");
        let ams_handle = thread::Builder::new()
            .stack_size(DEFAULT_STACK)
            .spawn_with_priority(ThreadPriority::Max, move |_| {
                ams.service_function();
            });

        if let Ok(join_handle) = ams_handle {
            if join_handle.is_finished() {
                return Err(ErrorCode::AmsBoot);
            }
            //Build description and insert in env lock
            ams_aid.set_id(join_handle.thread().id());
            deck().write().add_ams(ams_aid, join_handle);
            Ok(())
        } else {
            Err(ErrorCode::AmsBoot)
        }
    }

    /// This method creates agents of the given `T` type that implements [`Behavior`]
    ///  with the specified values (nickname, priority, and stack size).
    ///  If successful, it will return a `Ok(aid)` with the [`Description`] of the agent.
    ///  This Agent is not active by default and must be started by [`start`](Self::start)
    pub fn add_agent<T: Behavior + AgentBuild + Send + 'static>(
        &self,
        nickname: &'static str,
        priority: u8,
        stack_size: usize,
    ) -> Result<Description, ErrorCode> {
        // check name
        if RESERVED_NAMES.contains(&nickname) {
            return Err(ErrorCode::InvalidName);
        }
        // build agent
        let hap = self.name;
        let (tx, rx) = sync_channel::<Message>(1);
        let mut aid = Description::new(nickname, hap, tx);
        let control_block = Arc::new(ControlBlock::default());
        let base_agent = Agent::new(nickname, hap, rx, control_block.clone());
        if deck().read().search_agent(&aid).is_ok() {
            return Err(ErrorCode::Duplicated);
        }

        // check prio
        if priority == ThreadPriorityValue::MAX {
            return Err(ErrorCode::InvalidPriority(
                "Max priority only allowed for Services",
            ));
        }
        let thread_priority =
            ThreadPriority::try_from(priority).map_err(ErrorCode::InvalidPriority)?;

        // spawn agent with spinlock
        let agent = T::agent_builder(base_agent);
        let agent_handle = thread::Builder::new()
            .stack_size(stack_size)
            .spawn_with_priority(ThreadPriority::Min, move |_| execute(agent));

        // register on env
        let join_handle = agent_handle.map_err(|_| ErrorCode::AgentPanic)?;

        //Build description and insert in env lock
        aid.set_id(join_handle.thread().id());
        deck()
            .write()
            .add_agent(aid.clone(), join_handle, thread_priority, control_block)?;
        Ok(aid)
    }

    /// This method creates agents of the given `T` type that implements [`Behavior`]
    ///  with the specified values (nickname, priority, and stack size, parameter).
    ///  If successful, it will return a `Ok(aid)` with the [`Description`] of the agent.
    ///  This Agent is not active by default and must be started by [`start`](Self::start)
    pub fn add_agent_with_param<T: Behavior + AgentBuildParam + Send + 'static>(
        &self,
        nickname: &'static str,
        priority: u8,
        stack_size: usize,
        param: T::Parameter,
    ) -> Result<Description, ErrorCode> {
        // check name
        if RESERVED_NAMES.contains(&nickname) {
            return Err(ErrorCode::InvalidName);
        }

        // build agent
        let hap = self.name;
        let (tx, rx) = sync_channel::<Message>(1);
        let mut aid = Description::new(nickname, hap, tx);
        let control_block = Arc::new(ControlBlock::default());
        let base_agent = Agent::new(nickname, hap, rx, control_block.clone());
        if deck().read().search_agent(&aid).is_ok() {
            return Err(ErrorCode::Duplicated);
        }


        // check prio
        if priority == ThreadPriorityValue::MAX {
            return Err(ErrorCode::InvalidPriority(
                "Max priority only allowed for Services",
            ));
        }
        let thread_priority =
            ThreadPriority::try_from(priority).map_err(ErrorCode::InvalidPriority)?;

        // spawn agent with spinlock
        let agent = T::agent_with_param_builder(base_agent, param);
        let agent_handle = thread::Builder::new()
            .stack_size(stack_size)
            .spawn_with_priority(ThreadPriority::Min, move |_| {
                execute(agent);
            });

        // register on env
        let join_handle = agent_handle.map_err(|_| ErrorCode::AgentPanic)?;

        //Build description and insert in env lock
        aid.set_id(join_handle.thread().id());
        deck()
            .write()
            .add_agent(aid.clone(), join_handle, thread_priority, control_block)?;
        Ok(aid)
    }

    /// Transition the agent from the initiated state into the active state, required for it to execute its behavior.
    pub fn start(&self, aid: &Description) -> Result<(), ErrorCode> {
        let guard = deck().read();
        let entry = guard.get_agent(aid)?;
        let thread = entry.thread();
        if thread
            .get_priority()
            .map_err(ErrorCode::AgentStart)?
            .eq(&ThreadPriority::Min)
        {
            return Err(ErrorCode::AgentPanic);
        }
        let priority = entry.priority();
        if let Err(error) = thread.set_priority(priority) {
            return Err(ErrorCode::AgentStart(error));
        }
        entry.control_block().active()
    }

    //COULD ADD PLATFORM FUNCTIONS AND CALL THEM FROM AMS AGENT
}

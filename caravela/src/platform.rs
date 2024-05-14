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
    platform::environment::{aid_from_thread, platform_env},
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
use thread_priority::{ThreadBuilderExt, ThreadExt, ThreadPriority};

use self::environment::{agent_thread_handle, aid_from_name};

//pub mod organization;
pub(crate) mod environment;

//static AMS_LOCK: OnceLock<Description> = OnceLock::new();

type SpinLockMap = HashMap<Description, (Arc<AtomicBool>, ThreadPriority)>;

#[derive(Debug)]
pub struct Platform {
    name: &'static str,
    //ams_aid: &'static OnceLock<Description>,
    deck: Arc<RwLock<Deck>>,
    spinlock_map: SpinLockMap,
}

impl Platform {
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

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn boot(&mut self) -> Result<(), ErrorCode> {
        let default: DefaultConditions = DefaultConditions;
        self.boot_with_ams_conditions(default)
    }

    pub fn boot_with_ams_conditions<T: UserConditions + Send + 'static>(
        &mut self,
        conditions: T,
    ) -> Result<(), ErrorCode> {
        //
        let (tx, rx) = sync_channel::<Message>(1);
        let mut ams_aid = Description::new("AMS", self.name(), tx);
        let mut ams = Ams::<T>::new(rx, self.deck.clone(), conditions);
        //
        if let Ok(mut env_guard) = platform_env().write() {
            let ams_handle = thread::Builder::new()
                .stack_size(DEFAULT_STACK)
                .spawn_with_priority(ThreadPriority::Max, move |_| {
                    if let Ok(aid) = aid_from_thread(thread::current().id()) {
                        ams.hub.set_aid(aid);
                        println!("\nBOOTING AMS: {}\n", ams.hub.aid());
                        ams.service_function();
                    } else {
                        panic!("Could not start AMS");
                    }
                    //ams.hub.set_aid(aid)
                    //println!("\nBOOTING AMS: {}\n", ams.hub.aid());
                    //ams.service_function();
                });
            if let Ok(handle) = ams_handle {
                if handle.is_finished() {
                    return Err(ErrorCode::AmsBoot);
                }
                //Build description and insert in env lock
                ams_aid.set_thread(handle.thread().id());
                env_guard.insert_service(ams_aid, handle);
                Ok(())
            } else {
                Err(ErrorCode::AmsBoot)
            }
        } else {
            Err(ErrorCode::PoisonedEnvironment)
        }
    }

    pub fn add<T: Behavior + Send + 'static>(
        &mut self,
        nickname: &'static str,
        priority: u8,
        stack_size: usize,
    ) -> Result<Description, ErrorCode> {
        // build agent
        let hap = self.name.clone();
        let (tx, rx) = sync_channel::<Message>(1);
        let mut aid = Description::new(nickname, hap, tx);
        let tcb = Arc::new(ControlBlock::default());
        let deck = self.deck.clone();
        let base_agent = Agent::new(rx, deck, tcb.clone());
        if aid_from_name(&base_agent.aid().name()) == Err(ErrorCode::AidHandleNone) {
            return Err(ErrorCode::Duplicated);
        }
        let agent = T::agent_builder(base_agent);

        //check prio
        let priority = match ThreadPriority::try_from(priority) {
            Ok(agent_priority) => agent_priority,
            Err(error) => return Err(ErrorCode::InvalidPriority(error)),
        };

        // spawn agent with spinlock
        let spinlock = Arc::new(AtomicBool::new(false));
        let spinlock_platform = spinlock.clone();
        if let Ok(mut env_guard) = platform_env().write() {
            let agent_handle = thread::Builder::new()
                .stack_size(stack_size)
                .spawn_with_priority(ThreadPriority::Min, move |_| {
                    while spinlock.load(Ordering::Acquire) != false {
                        hint::spin_loop();
                    }
                    execute(agent);
                });
            // register on env
            if let Ok(handle) = agent_handle {
                if handle.is_finished() {
                    return Err(ErrorCode::AgentLaunch);
                }
                //Build description and insert in env lock
                aid.set_thread(handle.thread().id());
                env_guard.insert_agent(aid.clone(), handle, tcb);
                self.spinlock_map
                    .insert(aid.clone(), (spinlock_platform, priority));
                //register on deck -> TODO MOVE TO INIT FUNC IN BEHAVIOR
                let _ = self.deck.write().unwrap().insert_agent(aid.clone());
                Ok(aid)
            } else {
                Err(ErrorCode::AgentLaunch)
            }
        } else {
            Err(ErrorCode::PoisonedEnvironment)
        }
    }

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

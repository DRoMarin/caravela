use crate::{
    deck::Deck,
    entity::{
        agent::{
            behavior::{execute, Behavior},
            Agent, ControlBlock,
        },
        service::{ams::Ams, DefaultConditions, Service},
        Description,
    },
    ErrorCode, Message,
};
use std::sync::{mpsc::sync_channel, Arc, OnceLock, RwLock};
use thread_priority::{ThreadBuilderExt, ThreadPriority};

//pub mod organization;
pub(crate) mod env {
    use std::{
        collections::HashMap,
        sync::{Arc, OnceLock, RwLock},
        thread::JoinHandle,
    };

    use crate::{agent::ControlBlock, Description};

    #[derive(Debug)]
    pub(crate) struct DirectoryEntry {
        pub(crate) join_handle: JoinHandle<()>,
        pub(crate) control_block: Arc<ControlBlock>,
    }

    pub(crate) type PlatformDirectory = HashMap<Description, DirectoryEntry>;
    pub(crate) type AidDirectory = HashMap<String, Description>;

    pub(crate) static PLATFORM_ENV: OnceLock<RwLock<PlatformDirectory>> = OnceLock::new();
    pub(crate) static AID_ENV: OnceLock<RwLock<AidDirectory>> = OnceLock::new();

    pub(crate) fn platform_env() -> &'static RwLock<PlatformDirectory> {
        PLATFORM_ENV.get_or_init(|| RwLock::new(PlatformDirectory::new()))
    }
    pub(crate) fn aid_env() -> &'static RwLock<AidDirectory> {
        AID_ENV.get_or_init(|| RwLock::new(AidDirectory::new()))
    }
}

static AMS_LOCK: OnceLock<Description> = OnceLock::new();

#[derive(Debug)]
pub struct Platform {
    pub(crate) name: String,
    pub(crate) ams_aid: &'static OnceLock<Description>,
    pub(crate) deck: Arc<RwLock<Deck>>,
}

impl Platform {
    pub fn new(name: String) -> Self {
        let deck = Arc::new(RwLock::new(Deck::new()));
        Self {
            name,
            ams_aid: &AMS_LOCK,
            deck,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn boot(&mut self) -> Result<(), ErrorCode> {
        let default: DefaultConditions = DefaultConditions;
        let (tx, rx) = sync_channel::<Message>(1);

        let mut ams = Ams::<DefaultConditions>::new(self.name(), rx, self.deck.clone(), default)?;

        self.ams_aid.set(ams.hub.aid().clone());

        let mut deck_guard = self.deck.write().unwrap();
        deck_guard
            .address_directory
            .insert(self.ams_aid.clone(), tx);
        //deck_guard
        //    .white_pages_directory
        //    .insert(ams_aid.nickname(), ams_aid.clone());
        //deck_guard.address_directory.insert(ams_aid.clone(), tx);

        let ams_handle = std::thread::Builder::new().spawn_with_priority(
            ThreadPriority::Crossplatform(ams.hub.resources().priority()),
            move |_| {
                println!("\nBOOTING AMS: {}\n", ams.hub.aid());
                ams.service_function();
            },
        );
        /*if ams_handle.is_finished() {
            return Err("AMS ended");
        }*/
        if let Ok(handle) = ams_handle {
            deck_guard.handle_directory.insert(ams_aid, handle);
            Ok(())
        } else {
            Err(ErrorCode::AmsBoot)
        }
    }

    pub fn add<T: Behavior>(
        &mut self,
        nickname: String,
        priority: u8,
        stack_size: usize,
    ) -> Result<T, ErrorCode> {
        let tcb = Arc::new(ControlBlock::default());
        let hap = self.name.clone();
        let (tx, rx) = sync_channel::<Message>(1);
        let deck = self.deck.clone();
        let base_agent_creation = Agent::new(
            nickname.clone(),
            priority,
            stack_size,
            rx,
            deck,
            tcb.clone(),
            hap,
        );
        let mut base_agent = base_agent_creation?;
        let mut deck_guard = self.deck.write().unwrap();

        deck_guard
            .control_block_directory
            .insert(base_agent.aid(), tcb);
        deck_guard
            .white_pages_directory
            .insert(base_agent.aid().nickname(), base_agent.aid());
        deck_guard.address_directory.insert(base_agent.aid(), tx);
        base_agent
            .directory
            .insert("AMS".to_string(), self.ams_aid.clone().unwrap());

        let agent = T::agent_builder(base_agent);
        Ok(agent)
    }

    pub fn start(&mut self, mut agent: impl Behavior + Send + 'static) -> Result<(), ErrorCode> {
        let aid = agent.agent_mut_ref().aid();
        let prio = agent.agent_mut_ref().resources().priority();
        let mut platform_guard = self.deck.write().unwrap();
        let agent_handle = std::thread::Builder::new()
            .spawn_with_priority(ThreadPriority::Crossplatform(prio), move |_| execute(agent));
        if let Ok(handle) = agent_handle {
            platform_guard.handle_directory.insert(aid, handle);
        } else {
            return Err(ErrorCode::AgentLaunch);
        }
        Ok(())
    }
    //COULD ADD PLATFORM FUNCTIONS AND CALL THEM FROM AMS AGENT
}

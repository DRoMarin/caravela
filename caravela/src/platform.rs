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
    ErrorCode,
};
use std::sync::{Arc, RwLock};
use thread_priority::{ThreadBuilderExt, ThreadPriority};

//pub mod organization;

#[derive(Debug)]
pub struct Platform {
    pub(crate) name: String,
    pub(crate) ams_aid: Option<Description>,
    pub(crate) deck: Arc<RwLock<Deck>>,
}

impl Platform {
    pub fn new(name: String) -> Self {
        let deck = Arc::new(RwLock::new(Deck::new()));
        Self {
            name,
            ams_aid: None,
            deck,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn boot(&mut self) -> Result<(), ErrorCode> {
        let ams_nickname = "AMS".to_string();
        let default = DefaultConditions;
        let mut ams = Ams::<DefaultConditions>::new(self.name(), self.deck.clone(), default)?;
        let mut deck_guard = self.deck.write().unwrap();

        deck_guard
            .white_pages_directory
            .insert(ams.hub.aid().name(), ams.hub.aid());
        self.ams_aid = Some(ams.hub.aid());

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
            deck_guard.handle_directory.insert(ams_nickname, handle);
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
        let deck = self.deck.clone();
        let base_agent_creation = Agent::new(
            nickname.clone(),
            priority,
            stack_size,
            deck,
            tcb.clone(),
            hap,
        );
        let mut base_agent = base_agent_creation?;
        let mut deck_guard = self.deck.write().unwrap();

        deck_guard
            .control_block_directory
            .insert(nickname.clone(), tcb);
        deck_guard
            .white_pages_directory
            .insert(base_agent.aid().nickname(), base_agent.aid());
        base_agent
            .directory
            .insert("AMS".to_string(), self.ams_aid.clone().unwrap());

        println!("{}", self.ams_aid.as_ref().unwrap());
        let agent = T::agent_builder(base_agent);
        Ok(agent)
    }

    pub fn start(&mut self, mut agent: impl Behavior + Send + 'static) -> Result<(), ErrorCode> {
        let nickname = agent.agent_mut_ref().aid().name();
        let prio = agent.agent_mut_ref().resources().priority();
        let mut platform_guard = self.deck.write().unwrap();
        let agent_handle = std::thread::Builder::new()
            .spawn_with_priority(ThreadPriority::Crossplatform(prio), move |_| execute(agent));
        if let Ok(handle) = agent_handle {
            platform_guard.handle_directory.insert(nickname, handle);
        } else {
            return Err(ErrorCode::AgentLaunch);
        }
        Ok(())
    }
    //COULD ADD PLATFORM FUNCTIONS AND CALL THEM FROM AMS AGENT
}

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
};
use std::sync::{atomic::AtomicBool, Arc, RwLock};
use thread_priority::*;

//pub mod organization;

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

    pub fn boot(&mut self) -> Result<(), &str> {
        let default = DefaultConditions;
        let mut ams = Ams::<DefaultConditions>::new(self, default);
        let ams_name = "AMS".to_string();
        let mut deck_guard = self.deck.write().unwrap();
        deck_guard
            .white_pages_directory
            .insert(ams_name.clone(), ams.hub.get_aid());
        self.ams_aid = Some(ams.hub.get_aid());
        let ams_handle = std::thread::Builder::new().spawn_with_priority(
            ThreadPriority::Crossplatform(ams.hub.get_resources().get_priority()),
            move |_| {
                println!("\nBOOTING AMS: {}\n", ams.hub.get_aid().get_name());
                ams.service_function();
            },
        );
        /*if ams_handle.is_finished() {
            return Err("AMS ended");
        }*/
        if let Ok(handle) = ams_handle {
            deck_guard.handle_directory.insert(ams_name, handle);
        }
        Ok(())
    }

    pub fn add<T: Behavior>(
        &mut self,
        nickname: String,
        priority: u8,
        stack_size: usize,
    ) -> Result<T, &str> {
        //) -> Result<Agent<T>, &str> {
        let tcb = Arc::new(ControlBlock {
            active: AtomicBool::new(false),
            wait: AtomicBool::new(false),
            suspend: AtomicBool::new(false),
            quit: AtomicBool::new(false),
        });
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
        match base_agent_creation {
            Ok(mut base_agent) => {
                let mut deck_guard = self.deck.write().unwrap();
                deck_guard
                    .control_block_directory
                    .insert(nickname.clone(), tcb);
                deck_guard
                    .white_pages_directory
                    .insert(nickname, base_agent.get_aid());
                base_agent
                    .directory
                    .insert("AMS".to_string(), self.ams_aid.clone().unwrap());
                let agent = T::agent_builder(base_agent);
                Ok(agent)
            }
            Err(x) => Err(x),
        }
    }

    pub fn start(&mut self, mut agent: impl Behavior + Send + 'static) -> Result<(), &str> {
        let nickname = agent.get_agent_ref().get_nickname();
        let prio = agent.get_agent_ref().get_resources().get_priority();
        let mut platform_guard = self.deck.write().unwrap();
        let agent_handle = std::thread::Builder::new()
            .spawn_with_priority(ThreadPriority::Crossplatform(prio), move |_| execute(agent));
        if let Ok(handle) = agent_handle {
            platform_guard.handle_directory.insert(nickname, handle);
        } else {
            return Err("Could not launch agent");
        }
        Ok(())
    }
    //COULD ADD PLATFORM FUNCTIONS AND CALL THEM FROM AMS AGENT
}
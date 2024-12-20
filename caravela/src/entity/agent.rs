/// A collection of traits that give agents their behavior and formally defines them as agents.
pub mod behavior;

use crate::{
    deck::deck,
    entity::{
        //messaging::{Content, Message, MessageType, RequestType, SyncType},
        messaging::{ActionType, Content, Message, MessageType, SyncType},
        Description,
        Hub,
    },
    ErrorCode, Rx, MAX_SUBSCRIBERS,
};
use std::{
    collections::HashMap,
    fmt::Display,
    hint,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

type ContactList = HashMap<String, Description>;

/// The different states in an Agent Lifecycle.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum AgentState {
    /// The agent is present in the platform, but inactive.
    #[default]
    Initiated,
    /// The agent is running.
    Active,
    /// The agent is temporarily halted.
    Waiting,
    /// The agent is indifinately unavailable.
    Suspended,
    /// The agent is finished
    Terminated,
}

impl Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentState::Initiated => write!(f, "Initiated"),
            AgentState::Active => write!(f, "Active"),
            AgentState::Waiting => write!(f, "Waiting"),
            AgentState::Suspended => write!(f, "Suspended"),
            AgentState::Terminated => write!(f, "Terminated"),
        }
    }
}

impl From<usize> for AgentState {
    fn from(value: usize) -> Self {
        match value {
            0 => AgentState::Initiated,
            1 => AgentState::Active,
            2 => AgentState::Waiting,
            3 => AgentState::Suspended,
            4 => AgentState::Terminated,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct ControlBlock {
    state: AtomicUsize,
}

pub(crate) type ControlBlockArc = Arc<ControlBlock>;

impl ControlBlock {
    pub(crate) fn agent_state(&self) -> AgentState {
        self.state.load(Ordering::Relaxed).into()
    }
    fn set_state(&self, state: AgentState) {
        self.state.store(state as usize, Ordering::Relaxed);
    }
    pub(crate) fn quit(&self) -> Result<(), ErrorCode> {
        let current = self.agent_state();
        let target = AgentState::Terminated;
        current
            .eq(&AgentState::Active)
            .then(|| self.set_state(target))
            .ok_or(ErrorCode::InvalidStateChange(current, target))
    }
    pub(crate) fn suspend(&self) -> Result<(), ErrorCode> {
        let current = self.agent_state();
        let target = AgentState::Suspended;
        current
            .eq(&AgentState::Active)
            .then(|| self.set_state(target))
            .ok_or(ErrorCode::InvalidStateChange(current, target))
    }
    pub(crate) fn wait(&self) {
        self.set_state(AgentState::Waiting);
    }
    pub(crate) fn active(&self) -> Result<(), ErrorCode> {
        let current = self.agent_state();
        let target = AgentState::Active;
        { current.ne(&AgentState::Active) && current.ne(&AgentState::Terminated) }
            .then(|| self.set_state(target))
            .ok_or(ErrorCode::InvalidStateChange(current, target))
    }
}

/// The base agent type with AID, life cycle control, and messaging functionality.
#[derive(Debug)]
pub struct Agent {
    nickname: &'static str,
    hap: &'static str,
    hub: Hub,
    directory: ContactList,
    control_block: ControlBlockArc,
    //pub membership,
}

impl Agent {
    pub(crate) fn new(
        nickname: &'static str,
        hap: &'static str,
        rx: Rx,
        control_block: ControlBlockArc,
    ) -> Self {
        let directory: ContactList = HashMap::with_capacity(MAX_SUBSCRIBERS);
        let hub = Hub::new(rx);
        Self {
            nickname,
            hap,
            hub,
            directory,
            control_block,
        }
    }
    /// Get the Agent's name as the formated string `nickname@hap`.
    pub fn name(&self) -> String {
        format!("{}@{}", self.nickname, self.hap)
    }
    /// Get the Agent Identifier Description (AID) of the agent as [`Description`].
    pub fn aid(&self) -> Result<Description, ErrorCode> {
        deck().read().get_aid_from_thread(thread::current().id())
    }

    /// Send a [`Message`] with the desired [`MessageType`] and [`Content`] to the target agent.
    /// The receiver shall be addressed by its nickname, if a [`Description`] is to be used, employ [`self.send_to_aid`] instead.
    //TBD: add block/nonblock parameter
    pub fn send_to(
        &self,
        nickname: &str,
        message_type: MessageType,
        content: Content,
        //content: String,
    ) -> Result<(), ErrorCode> {
        let agent_aid = if let Some(agent_aid) = self.directory.get(nickname) {
            agent_aid.to_owned()
        } else {
            //only looking for local agents
            let name = self.fmt_local_agent(nickname);
            deck().read().get_aid_from_name(&name)?
        };
        self.send_to_aid(agent_aid, message_type, content)
    }

    /// Send a [`Message`] with the desired [`MessageType`] and [`Content`] to the target agent.
    /// The agent shall be addressed by its [`Description`].
    pub fn send_to_aid(
        &self,
        aid: Description,
        message_type: MessageType,
        content: Content,
        //content: String,
    ) -> Result<(), ErrorCode> {
        let msg = Message::new(self.aid()?, aid, message_type, content);
        self.hub.send(msg, SyncType::Blocking)
    }

    /// Send a [`Message`] with the desired [`MessageType`] and [`Content`] to all the agents in the contact list.
    pub fn send_to_all(
        &self,
        message_type: MessageType,
        content: Content,
        //content: String,
    ) -> Result<(), ErrorCode> {
        let agents = self.directory.values();
        for aid in agents {
            self.send_to_aid(aid.clone(), message_type.clone(), content.clone())?;
        }
        Ok(())
    }

    /// Wait for a [`Message`] to arrive. This operation blocks the agent.
    pub fn receive(&self) -> Result<Message, ErrorCode> {
        caravela_messaging!("{}: waiting for message", self.name());
        self.hub.receive().map(|x| {
            caravela_messaging!("{}: message received!", self.name());
            x
        })
    }

    /// Add an agent to the contact list. The target agent needs to be addressed by its nickname.
    pub fn add_contact(&mut self, nickname: &str) -> Result<(), ErrorCode> {
        //only looking for local agents
        let name = self.fmt_local_agent(nickname);
        let agent = deck().read().get_aid_from_name(&name)?;
        self.add_contact_aid(nickname, agent)
    }

    /// Add a contact to the contact list. The target agent needs to be addressed by its [`Description`].
    pub fn add_contact_aid(
        &mut self,
        nickname: &str,
        description: Description,
    ) -> Result<(), ErrorCode> {
        if self.directory.len().eq(&MAX_SUBSCRIBERS) {
            Err(ErrorCode::ListFull)
        } else if self.directory.contains_key(nickname) {
            Err(ErrorCode::Duplicated)
        } else {
            self.directory.insert(nickname.to_string(), description);
            Ok(())
        }
    }

    pub(crate) fn fmt_local_agent(&self, nickname: &str) -> String {
        format!("{nickname}@{}", self.hap)
    }

    pub(crate) fn init(&self) {
        while self.control_block.agent_state() == AgentState::Initiated {
            hint::spin_loop()
        }
    }

    /// Halt the agent's operation for a specified duration of time in milliseconds.
    pub fn wait(&self, time: u64) {
        let dur = Duration::from_millis(time); //TBD could remove
        self.control_block.wait();
        caravela_status!("{}: Waiting", self.name());
        thread::park_timeout(dur);
        caravela_status!("{}: Resuming", self.name());
        if self.control_block.agent_state().ne(&AgentState::Active) {
            let _ = self.control_block.active();
        }
    }

    pub(crate) fn suspend(&self) {
        if self.control_block.agent_state().eq(&AgentState::Suspended) {
            caravela_status!("{}: Suspending", self.name());
            thread::park();
            caravela_status!("{}: Resuming", self.name());
        }
    }

    pub(crate) fn quit(&self) -> bool {
        self.control_block
            .agent_state()
            .eq(&AgentState::Terminated)
            .then(|| caravela_status!("{}: Terminating", self.name()))
            .is_some()
    }

    pub(crate) fn takedown(&self) -> Result<(), ErrorCode> {
        //let ams = deck().read().get_ams_address_for_hap(&self.hap)?;
        let ams = deck().read().ams_aid().clone();
        let msg_type = MessageType::Request;
        let msg_content = Content::Action(ActionType::Deregister(self.aid()?));
        //let msg_content = format!("deregister {}", self.aid()?.name());
        self.send_to_aid(ams, msg_type, msg_content)?;
        self.receive().map(|_| {
            caravela_status!("{}: Terminating", self.name());
        })
    }
}

/// This trait defines how an agent without patameters must be built by the platform.
pub trait AgentBuild {
    /// Required function to build the derived agent instance without a parameter field.
    fn agent_builder(base_agent: Agent) -> Self;
}

/// This trait defines how an agent with patameters must be built by the platform.
pub trait AgentBuildParam {
    /// Associated parameter type
    type Parameter;
    /// Required function to build the derived agent instance with a parameter field.
    fn agent_with_param_builder(base_agent: Agent, param: Self::Parameter) -> Self;
}

/// A collection of traits that give agents their behavior and formally defines them as agents.
pub mod behavior;

use crate::{
    deck::DeckAccess,
    entity::{
        messaging::{Content, Message, MessageType, RequestType},
        Description, Entity, Hub,
    },
    ErrorCode, MAX_SUBSCRIBERS, RX,
};
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, current},
    time::Duration,
};

type ContactList = HashMap<String, Description>;

/// The different states in an Agent Lifecycle.
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub enum AgentState {
    /// The agent is present in the platform, but inactive.
    #[default]
    Initiated,
    /// THe agent is Active
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

#[derive(Debug, Default)]
pub(crate) struct ControlBlock {
    pub active: AtomicBool,
    pub wait: AtomicBool,
    pub suspend: AtomicBool,
    pub quit: AtomicBool,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ControlBlockAccess(pub(crate) Arc<ControlBlock>);

impl ControlBlockAccess {
    pub(crate) fn agent_state(&self) -> AgentState {
        if self.quit().load(Ordering::Relaxed) {
            AgentState::Terminated
        } else if self.suspend().load(Ordering::Relaxed) {
            AgentState::Suspended
        } else if self.wait().load(Ordering::Relaxed) {
            AgentState::Waiting
        } else if self.active().load(Ordering::Relaxed) {
            AgentState::Active
        } else {
            AgentState::Initiated
        }
    }

    pub(crate) fn quit(&self) -> &AtomicBool {
        &self.0.quit
    }
    pub(crate) fn suspend(&self) -> &AtomicBool {
        &self.0.suspend
    }
    pub(crate) fn wait(&self) -> &AtomicBool {
        &self.0.wait
    }
    pub(crate) fn active(&self) -> &AtomicBool {
        &self.0.active
    }
}

/// The base agent type with AID, life cycle control, and messaging functionality.
#[derive(Debug)]
pub struct Agent {
    hub: Hub,
    directory: ContactList,
    tcb: ControlBlockAccess,
    //pub membership,
}

impl Entity for Agent {
    fn set_aid(&mut self, aid: Description) {
        self.hub.set_aid(aid);
    }
}

impl Agent {
    pub(crate) fn new(
        //name: String,
        aid: Description,
        rx: RX,
        deck: DeckAccess,
        tcb: ControlBlockAccess,
    ) -> Self {
        let directory: ContactList = HashMap::with_capacity(MAX_SUBSCRIBERS);
        let hub = Hub::new(aid, rx, deck);
        Self {
            hub,
            directory,
            tcb,
        }
    }
    /// Get the Agent Identifier Description (AID) of the agent as [`Description`].
    pub fn aid(&self) -> &Description {
        self.hub.aid()
    }

    /// Get the [`Message`] currently held by the agent.
    pub fn msg(&self) -> Message {
        self.hub.msg()
    }

    /// Set the [`Content`] and [`MessageType`] of the message. This is used to format the message before it is sent.
    pub fn set_msg(&mut self, msg_type: MessageType, msg_content: Content) {
        self.hub.set_msg(msg_type, msg_content)
    }

    /// Send the currently held message to the target agent. The receiver needs to be addressed by its nickname.
    //TBD: add block/nonblock parameter
    pub fn send_to(&mut self, agent: &str) -> Result<(), ErrorCode> {
        let agent_aid = if let Some(agent_aid) = self.directory.get(agent) {
            agent_aid.to_owned()
        } else {
            //only looking for local agents
            let name = self.fmt_local_agent(agent);
            self.hub.deck()?.get_aid_from_name(&name)?
        };
        self.send_to_aid(agent_aid)
    }

    /// Send the currently held [`Message`] to the target agent. The agent needs to be addressed by its [`Description`].
    pub fn send_to_aid(&mut self, aid: Description) -> Result<(), ErrorCode> {
        self.hub.set_msg_receiver(aid);
        self.hub.set_msg_sender(self.aid().clone());
        self.hub.send()
    }

    /// Send the currently held ['Message'] to all the agents in the contact list.
    pub fn send_to_all(&mut self) -> Result<(), ErrorCode> {
        self.hub.set_msg_sender(self.aid().clone());
        let agents = self.directory.values();
        for aid in agents {
            self.hub.set_msg_receiver(aid.clone());
            self.hub.send()?;
        }
        Ok(())
    }

    /// Wait for a [`Message`] to arrive. This operation blocks the agent and will overwrite the currently held [`Message`].
    pub fn receive(&mut self) -> Result<MessageType, ErrorCode> {
        caravela_messaging!("{}: waiting for message", self.aid());
        self.hub.receive().map(|x| {
            caravela_messaging!("{}: message received!", self.aid());
            x
        })
    }

    /// Add an agent to the contact list. The target agent needs to be addressed by its nickname.
    pub fn add_contact(&mut self, nickname: &str) -> Result<(), ErrorCode> {
        //only looking for local agents
        let name = self.fmt_local_agent(nickname);
        let agent = self.hub.deck()?.get_aid_from_name(&name)?;
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

    /// Halt the agent's operation for a specified duration of time in milliseconds.
    pub fn wait(&self, time: u64) {
        self.tcb.wait().store(true, Ordering::Relaxed);
        caravela_status!("{}: Waiting", self.aid());
        let dur = Duration::from_millis(time); //TBD could remove
        thread::sleep(dur);
        self.tcb.wait().store(false, Ordering::Relaxed);
        caravela_status!("{}: Active", self.aid());
    }

    /*pub(crate) fn set_thread(&mut self) {
        self.hub.set_thread();
    }*/
    pub(crate) fn fmt_local_agent(&self, nickname: &str) -> String {
        let mut name = String::new();
        name.push_str(nickname);
        name.push('@');
        name.push_str(self.aid().hap());
        name
    }

    pub(crate) fn init(&mut self) {
        self.hub.set_thread(current().id());
        //TBD
    }

    pub(crate) fn suspend(&self) {
        if self.tcb.suspend().load(Ordering::Relaxed) {
            self.tcb.suspend().store(true, Ordering::Relaxed);
            caravela_status!("{}: Suspending", self.aid());
            thread::park();
            caravela_status!("{}: Resuming", self.aid());
            self.tcb.suspend().store(false, Ordering::Relaxed);
        }
    }

    pub(crate) fn quit(&self) -> bool {
        self.tcb.quit().load(Ordering::Relaxed)
    }

    pub(crate) fn takedown(&mut self) -> Result<(), ErrorCode> {
        let msg_type = MessageType::Request;
        let msg_content = Content::Request(RequestType::Deregister(self.aid().clone()));
        self.set_msg(msg_type, msg_content);

        let ams = self.hub.deck_mut()?.ams_aid().clone();
        {
            self.send_to_aid(ams)?;
        };
        self.receive().map(|_| {
            caravela_status!("{}: Terminated", self.aid());
        })
    }
}

/// This trait gives the platform access to the base agent element. It is required to execute the agent life cycle accordingly.
pub trait AgentBase {
    /// Required function to access  [`Agent`] base functionality.
    fn agent(&mut self) -> &mut Agent;
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

    /// Required function to give the platform access to parameter field.
    fn param(&mut self) -> &mut Self::Parameter;
}

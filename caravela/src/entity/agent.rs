pub use behavior::Behavior;
pub(crate) mod behavior;

use crate::{
    deck::Deck,
    entity::{
        messaging::{Content, Message, MessageType, RequestType},
        Description, Hub,
    },
    platform::environment::{aid_from_name, aid_from_thread},
    ErrorCode, MAX_SUBSCRIBERS, RX,
};
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};

type ContactList = HashMap<String, Description>;

/// The different states in an Agent Lifecycle.
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub enum AgentState {
    /// The Agent is present in the platform, but inactive.
    #[default]
    Initiated,
    /// THe Agent is Active
    Active,
    /// The Agent is temporarily halted.
    Waiting,
    /// The Agent is indifinately unavailable.
    Suspended,
    /// The Agent is finished
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

/// The base agent type with AID, task control, and messaging functionality.
#[derive(Debug)]
pub struct Agent {
    pub(crate) hub: Hub,
    pub(crate) directory: ContactList,
    pub(crate) tcb: Arc<ControlBlock>,
    //pub membership,
}

impl Agent {
    pub(crate) fn new(
        //name: String,
        rx: RX,
        deck: Arc<RwLock<Deck>>,
        tcb: Arc<ControlBlock>,
    ) -> Self {
        let directory: ContactList = HashMap::with_capacity(MAX_SUBSCRIBERS);
        let hub = Hub::new(rx, deck);
        Self {
            hub,
            directory,
            tcb,
        }
    }
    /// Get the current Agent's Agent Identifier Description (AID) struct.
    pub fn aid(&self) -> Description {
        self.hub.aid()
    }

    /// Get the Message struct currently held by the Agent.
    pub fn msg(&self) -> Message {
        self.hub.msg()
    }

    /// Set the contents and type of the message. This is used to format the message before it is sent.
    pub fn set_msg(&mut self, msg_type: MessageType, msg_content: Content) {
        self.hub.set_msg(msg_type, msg_content)
    }

    /// Send the currently held message to the target Agent. The Agent needs to be addressed by its AID struct.
    //TBD: add block/nonblock parameter
    pub fn send_to(&mut self, agent: &str) -> Result<(), ErrorCode> {
        println!("[INFO] {}: Sending {} to {}", self.aid(),self.hub.msg().message_type(), agent);
        let agent_aid = if let Some(agent_aid) = self.directory.get(agent) {
            agent_aid.to_owned()
        } else {
            //only looking for local agents
            let name = self.fmt_local_agent(agent);
            aid_from_name(&name)?
        };
        self.send_to_aid(&agent_aid)
    }

    /// Send the currently held message to the target Agent. The Agent needs to be addressed by its nickname.
    pub fn send_to_aid(&mut self, description: &Description) -> Result<(), ErrorCode> {
        self.hub.send_to_aid(description)
    }

    /// Wait for a messsage to arrive. This operation blocks the Agent and will overwrite the currently held Message.
    pub fn receive(&mut self) -> Result<MessageType, ErrorCode> {
        self.hub.receive()
    }

    /// Add a contact to the contact list. The target Agent needs to be addressed by its nickname.
    pub fn add_contact(&mut self, nickname: &str) -> Result<(), ErrorCode> {
        let msg_type = MessageType::Request;
        //only looking for local agents
        let name = self.fmt_local_agent(nickname);
        let agent = aid_from_name(&name)?;
        let msg_content = Content::Request(RequestType::Search(agent));
        self.set_msg(msg_type, msg_content);
        self.send_to("AMS")?;
        let msg_type = self.receive()?;
        match msg_type {
            MessageType::Inform => {
                if let Content::AmsAgentDescription(ams_agent_description) = self.msg().content() {
                    self.add_contact_aid(nickname, ams_agent_description.aid().clone())?;
                    Ok(())
                } else {
                    Err(ErrorCode::InvalidContent)
                }
            }
            MessageType::Failure => Err(ErrorCode::NotRegistered),

            _ => Err(ErrorCode::InvalidMessageType),
        }
    }

    /// Add a contact to the contact list. The target Agent needs to be addressed by its Description.
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

    /// Halt the Agent Behavior for a specified duration of time.
    pub fn wait(&self, time: u64) {
        self.tcb.wait.store(true, Ordering::Relaxed);
        let dur = Duration::from_millis(time); //TBD could remove
        thread::sleep(dur);
        self.tcb.wait.store(false, Ordering::Relaxed);
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

    pub(crate) fn init(&mut self) -> bool {
        if let Ok(aid) = aid_from_thread(thread::current().id()) {
            self.hub.set_aid(aid);
            println!("[INFO] {}: Starting", self.aid());
            self.tcb.active.store(true, Ordering::Relaxed);
            true
        } else {
            self.takedown();
            false
        }
    }

    pub(crate) fn suspend(&mut self) {
        if self.tcb.suspend.load(Ordering::Relaxed) {
            self.tcb.suspend.store(true, Ordering::Relaxed);
            thread::park();
            self.tcb.suspend.store(false, Ordering::Relaxed);
        }
    }

    pub(crate) fn quit(&self) -> bool {
        self.tcb.quit.load(Ordering::Relaxed)
    }

    pub(crate) fn takedown(&mut self) -> bool {
        let ams = "AMS";
        let msg_type = MessageType::Request;
        let msg_content = Content::Request(RequestType::Deregister(self.aid()));
        self.set_msg(msg_type, msg_content);
        let _ = self.send_to(ams);
        true
    }
}

pub use behavior::Behavior;
pub(crate) mod behavior;

use crate::{
    deck::{Deck, Directory},
    entity::{
        messaging::{Content, Message, MessageType, RequestType},
        Description, ExecutionResources, Hub,
    },
    ErrorCode, StackSize, MAX_PRIORITY, MAX_SUBSCRIBERS,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};

/// This Enum specifies the different states in an Agent Lifecycle.
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
    pub(crate) directory: Directory,
    pub(crate) tcb: Arc<ControlBlock>,
    //pub membership,
}

impl Agent {
    pub(crate) fn new(
        nickname: String,
        priority: u8,
        stack_size: StackSize,
        deck: Arc<RwLock<Deck>>,
        tcb: Arc<ControlBlock>,
        hap: String,
    ) -> Result<Self, &'static str> {
        if priority > (MAX_PRIORITY - 1) {
            return Err("Priority value invalid");
        };
        let directory: Directory = HashMap::with_capacity(MAX_SUBSCRIBERS);
        let resources = ExecutionResources::new(priority, stack_size);
        let hub = Hub::new(nickname, resources, deck, hap);
        Ok(Self {
            hub,
            directory,
            tcb,
        })
    }
    /// Get the current Agent's Agent Identifier Description (AID) struct.
    pub fn aid(&self) -> Description {
        self.hub.aid()
    }

    /// Get the Execution Resources struct of the current Agent.
    pub fn resources(&self) -> ExecutionResources {
        self.hub.resources()
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
        if let Some(agent) = self.directory.get(agent) {
            self.hub.send_to_aid(agent.clone())
        } else {
            self.hub.send_to(agent)
        }
    }

    /// Send the currently held message to the target Agent. The Agent needs to be addressed by its nickname.
    pub fn send_to_aid(&mut self, description: Description) -> Result<(), ErrorCode> {
        self.hub.send_to_aid(description)
    }

    /// Wait for a messsage to arrive. This operation blocks the Agent and will overwrite the currently held Message.
    pub fn receive(&mut self) -> Result<MessageType, ErrorCode> {
        self.hub.receive()
    }

    /// Add a contact to the contact list. The target Agent needs to be addressed by its nickname.
    pub fn add_contact(&mut self, agent: &str) -> Result<(), ErrorCode> {
        let msg_type = MessageType::Request;
        let msg_content = Content::Request(RequestType::Search(agent.to_string()));
        self.set_msg(msg_type, msg_content);
        let send_result = self.send_to("AMS");
        send_result?;
        let recv_result = self.receive();
        if let Ok(msg_type) = recv_result {
            match msg_type {
                MessageType::Inform => {
                    if let Content::AID(x) = self.msg().content() {
                        self.add_contact_aid(agent, x)
                    } else {
                        Err(ErrorCode::Invalid)
                    }
                }
                MessageType::Failure => Err(ErrorCode::NotRegistered),

                _ => Err(ErrorCode::Invalid),
            }
        } else {
            Err(ErrorCode::Disconnected)
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
        let dur = Duration::from_millis(time);
        thread::sleep(dur);
        self.tcb.wait.store(false, Ordering::Relaxed);
    }

    pub(crate) fn set_thread(&mut self) {
        self.hub.set_thread();
    }

    pub(crate) fn init(&mut self) -> bool {
        //println!("{}: STARTING", self.get_nickname());
        println!("{}: STARTING", self.aid());
        self.tcb.active.store(true, Ordering::Relaxed);
        true
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
        let ams = "AMS".to_string();
        let msg_type = MessageType::Request;
        //let msg_content = Content::Request(RequestType::Deregister(self.get_nickname()));
        let msg_content = Content::Request(RequestType::Deregister(self.aid().name()));
        self.set_msg(msg_type, msg_content);
        let _ = self.send_to(&ams);
        true
    }
}

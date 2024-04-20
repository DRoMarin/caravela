use crate::platform::{
    agent::Agent,
    entity::{
        messaging::{Content, Message, MessageType, RequestType},
        Description, ExecutionResources,
    },
    ErrorCode,
};
use private::ControlToken;
use std::{
    sync::{atomic::Ordering, mpsc::RecvError},
    thread,
    time::Duration,
};

mod private {
    use crate::platform::agent::Agent;
    pub struct ControlToken;
    pub trait SealedBehavior {}
    impl<T> SealedBehavior for Agent<T> {}
}

pub trait AgentBehavior: private::SealedBehavior {
    //this trait will define top level gets and actions like recv and send msg
    fn get_aid(&self) -> Description;
    fn get_nickname(&self) -> String;
    fn get_hap(&self) -> String;
    fn get_resources(&self) -> ExecutionResources;
    fn get_msg(&self) -> Message;
    fn set_msg(&mut self, msg_type: MessageType, msg_content: Content);
    fn send_to(&mut self, agent: &str) -> Result<(), ErrorCode>;
    fn send_to_aid(&mut self, description: Description) -> Result<(), ErrorCode>;
    //fn send_to_with_timeout(&mut self, agent: &str, timeout: u64) -> ErrorCode;
    //fn send_to_all(&self) -> ErrorCode;
    fn receive(&mut self) -> Result<MessageType, RecvError>;
    //fn get_thread_id(&self) -> Option<ID>;
}

pub trait AgentControl {
    //TBD
    fn init(&mut self, _: &private::ControlToken) -> bool;
    fn set_thread(&mut self, _: &private::ControlToken);
    fn suspend(&mut self, _: &private::ControlToken);
    fn wait(&self, time: u64);
    fn quit(&self, _: &private::ControlToken) -> bool;
    fn takedown(&mut self, _: &private::ControlToken) -> bool;
}

impl<T> AgentBehavior for Agent<T> {
    fn get_aid(&self) -> Description {
        self.hub.get_aid()
    }

    fn get_nickname(&self) -> String {
        self.hub.get_nickname()
    }

    fn get_hap(&self) -> String {
        self.hub.get_hap()
    }

    fn get_resources(&self) -> ExecutionResources {
        self.hub.get_resources()
    }

    fn get_msg(&self) -> Message {
        self.hub.get_msg()
    }

    fn set_msg(&mut self, msg_type: MessageType, msg_content: Content) {
        self.hub.set_msg(msg_type, msg_content)
    }

    //TBD: add block/nonblock parameter
    fn send_to(&mut self, agent: &str) -> Result<(), ErrorCode> {
        if let Some(agent) = self.directory.get(agent) {
            self.hub.send_to_aid(agent.clone())
        } else {
            self.hub.send_to(agent)
        }
    }

    fn send_to_aid(&mut self, description: Description) -> Result<(), ErrorCode> {
        self.hub.send_to_aid(description)
    }

    fn receive(&mut self) -> Result<MessageType, RecvError> {
        self.hub.receive()
    }
    /*fn receive_timeout(&mut self, timeout: u64) -> MessageType */
}

impl<T> AgentControl for Agent<T> {
    fn wait(&self, time: u64) {
        self.tcb.wait.store(true, Ordering::Relaxed);
        let dur = Duration::from_millis(time);
        thread::sleep(dur);
        self.tcb.wait.store(false, Ordering::Relaxed);
    }

    fn set_thread(&mut self, _: &private::ControlToken) {
        self.hub.set_thread();
    }

    fn init(&mut self, _: &private::ControlToken) -> bool {
        println!("{}: STARTING", self.get_nickname());
        self.tcb.active.store(true, Ordering::Relaxed);
        true
    }

    fn suspend(&mut self, _: &private::ControlToken) {
        if self.tcb.suspend.load(Ordering::Relaxed) {
            self.tcb.suspend.store(true, Ordering::Relaxed);
            thread::park();
            self.tcb.suspend.store(false, Ordering::Relaxed);
        }
    }

    fn quit(&self, _: &private::ControlToken) -> bool {
        self.tcb.quit.load(Ordering::Relaxed)
    }

    fn takedown(&mut self, _: &private::ControlToken) -> bool {
        let ams = "AMS".to_string();
        let msg_type = MessageType::Request;
        let msg_content = Content::Request(RequestType::Deregister(self.get_nickname()));
        self.set_msg(msg_type, msg_content);
        let _ = self.send_to(&ams);
        true
    }
}

pub trait Behavior: AgentControl + AgentBehavior {
    //: Entity {
    fn setup(&mut self) {
        println!("{}: no setup implemented", self.get_nickname());
    }

    fn done(&mut self) -> bool {
        println!("{}: execution done, taking down...", self.get_nickname());
        true
    }

    fn action(&mut self) {
        println!("{}: no action implemented", self.get_nickname());
    }

    fn failure_detection(&mut self) -> bool {
        println!("{}: no failure detection implemented", self.get_nickname());
        false
    }

    fn failure_identification(&mut self) {
        println!(
            "{}: no failure identification implemented",
            self.get_nickname()
        );
    }

    fn failure_recovery(&mut self) {
        println!("{}: no failure recovery implemented", self.get_nickname());
    }
}

pub(crate) fn execute(mut behavior: impl Behavior) {
    let token = ControlToken;
    behavior.set_thread(&token);
    if behavior.init(&token) {
        behavior.setup();
        loop {
            behavior.suspend(&token);
            if behavior.quit(&token) {
                break;
            }
            behavior.action();
            if behavior.failure_detection() {
                behavior.failure_identification();
                behavior.failure_recovery();
            }
            if behavior.done() {
                behavior.takedown(&token);
                break;
            }
        }
    }
}

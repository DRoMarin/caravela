use crate::platform::entity::Entity;
use private::TaskControl;

pub(crate) mod private {
    use crate::platform::{
        agent::Agent,
        entity::{
            messaging::{Content, MessageType, RequestType},
            Entity,
        },
        AgentState,
    };
    use std::{sync::atomic::Ordering, thread, time::Duration};

    pub trait TaskControl {
        //TBD
        fn init(&mut self) -> bool;
        fn set_thread(&mut self);
        fn suspend(&mut self);
        fn wait(&self, time: u64);
        fn quit(&self) -> bool;
        fn takedown(&mut self) -> bool;
    }

    impl<T> TaskControl for Agent<T> {
        fn set_thread(&mut self) {
            self.hub.aid.set_thread();
        }
        fn init(&mut self) -> bool {
            println!("{}: Resgistering", self.get_nickname());
            let ams = "AMS".to_string(); // + "@" + &self.hub.hap;
            self.msg.set_type(MessageType::Request);
            self.msg.set_content(Content::Request(RequestType::Register(
                self.get_nickname(),
                self.get_aid(),
            )));
            self.send_to(&ams);
            self.hub.tcb.init.wait();
            self.receive();
            true
        }
        fn suspend(&mut self) {
            if self.hub.tcb.suspend.load(Ordering::Relaxed) {
                {
                    self.hub.tcb.suspend.store(false, Ordering::Relaxed);
                    self.hub
                        .platform
                        .write()
                        .unwrap()
                        .state_directory
                        .entry(self.get_nickname())
                        .and_modify(|s| *s = AgentState::Suspended);
                }
                thread::park();
                {
                    self.hub.tcb.suspend.store(false, Ordering::Relaxed);
                    self.hub
                        .platform
                        .write()
                        .unwrap()
                        .state_directory
                        .entry(self.get_nickname())
                        .and_modify(|s| *s = AgentState::Active);
                }
            }
        }
        fn wait(&self, time: u64) {
            {
                self.hub
                    .platform
                    .write()
                    .unwrap()
                    .state_directory
                    .entry(self.get_nickname())
                    .and_modify(|s| *s = AgentState::Waiting);
            }
            let dur = Duration::from_millis(time);
            thread::sleep(dur);
        }
        fn quit(&self) -> bool {
            self.hub.tcb.quit.load(Ordering::Relaxed)
        }
        fn takedown(&mut self) -> bool {
            let ams = "AMS".to_string();
            self.msg.set_type(MessageType::Request);
            self.msg
                .set_content(Content::Request(RequestType::Deregister(
                    self.get_nickname(),
                )));
            self.send_to(&ams);
            true
        }
    }
}

pub trait Behavior: Entity {
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

pub(crate) fn execute(mut behavior: impl Behavior + TaskControl) {
    behavior.set_thread();
    if behavior.init() {
        behavior.setup();
        loop {
            behavior.suspend();
            if behavior.quit() {
                break;
            }
            behavior.action();
            if behavior.failure_detection() {
                behavior.failure_identification();
                behavior.failure_recovery();
            }
            if behavior.done() {
                behavior.takedown();
                break;
            }
        }
    }
}

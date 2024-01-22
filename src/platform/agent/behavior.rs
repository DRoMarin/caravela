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
            true
        }
        fn suspend(&mut self) {
            /*let suspend = self
                .hub
                .platform
                .read()
                .unwrap()
                .control_block_directory
                .get(&self.get_nickname())
                .unwrap()
                .suspend
                .load(Ordering::Relaxed);
            if suspend {
                {
                    let state = &mut self.hub.platform.write().unwrap().state_directory;
                    state
                        .entry(self.get_nickname())
                        .and_modify(|s| *s = AgentState::Suspended);
                }
                thread::park()
            }
            /*let platform = self.hub.platform.write().unwrap();
            let suspend = &platform
                .control_block_directory
                .get(&self.get_nickname())
                .as_mut()
                .unwrap()
                .suspend;

            if suspend.load(Ordering::Relaxed) {
                suspend.store(false, Ordering::Relaxed);
                {
                    //let mut state = self.hub.state_directory.as_ref().write().unwrap();
                    let state = &mut self.hub.platform.write().unwrap().state_directory;
                    state
                        .entry(self.hub.nickname.clone())
                        .and_modify(|s| *s = AgentState::Suspended);
                }
                thread::park();
            }
            //TO BE FIXED: THIS WILL (PROBABLY) NOT WORK*/*/
        }
        fn wait(&self, time: u64) {
            /*let state = &mut self.hub.platform.write().unwrap().state_directory;
            state
                .entry(self.get_nickname())
                .and_modify(|s| *s = AgentState::Waiting);*/
            let dur = Duration::from_millis(time);
            thread::sleep(dur);
        }
        fn quit(&self) -> bool {
            /*self.hub
            .platform
            .read()
            .unwrap()
            .control_block_directory
            .get(&self.get_nickname())
            .unwrap()
            .quit
            .load(Ordering::Relaxed)*/
            false
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
        print!("{}: no setup implemented\n", self.get_nickname());
    }
    fn done(&mut self) -> bool {
        println!("{}: execution done, taking down...\n", self.get_nickname());
        true
    }
    fn action(&mut self) {
        print!("{}: no action implemented\n", self.get_nickname());
    }
    fn failure_detection(&mut self) -> bool {
        println!(
            "{}: no failure detection implemented\n",
            self.get_nickname()
        );
        true
    }
    fn failure_identification(&mut self) {
        print!(
            "{}: no failure identification implemented\n",
            self.get_nickname()
        );
    }
    fn failure_recovery(&mut self) {
        print!("{}: no failure recovery implemented\n", self.get_nickname());
    }
}

pub(crate) fn execute(mut behavior: impl Behavior + TaskControl) {
    behavior.set_thread();
    let result = behavior.init();
    if result == true {
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

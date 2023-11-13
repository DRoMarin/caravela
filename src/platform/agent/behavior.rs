//use crate::platform::agent::Agent;
use private::TaskControl;

mod private {
    use std::{sync::atomic::Ordering, thread, time::Duration};

    use crate::platform::{agent::Agent, AgentState};

    pub(crate) trait TaskControl {
        //TBD
        fn init(&mut self) -> bool;
        fn set_thread(&mut self);
        fn suspend(&mut self);
        fn wait(&self, time: u64);
        fn quit(&self) -> bool;
        fn takedown(&self) {}
    }

    impl<T> TaskControl for Agent<T> {
        fn set_thread(&mut self) {
            self.hub.aid.set_thread();
        }
        fn init(&mut self) -> bool {
            //send register message

            //could change for recv message with accept or reject
            while !self
                .hub
                .control_block
                .as_ref()
                .unwrap()
                .init
                .load(Ordering::Relaxed)
            {
                //waiting
            }

            true
        }
        fn suspend(&mut self) {
            let suspend = &self.hub.control_block.as_ref().unwrap().suspend;
            if suspend.load(Ordering::Relaxed) {
                suspend.store(false, Ordering::Relaxed);
                {
                    let mut state = self.hub.state_directory.as_ref().unwrap().lock().unwrap();
                    state
                        .entry(self.hub.nickname.clone())
                        .and_modify(|s| *s = AgentState::Suspended);
                }
                thread::park();
            }
        }
        fn wait(&self, time: u64) {
            let mut state = self.hub.state_directory.as_ref().unwrap().lock().unwrap();
            state
                .entry(self.hub.nickname.clone())
                .and_modify(|s| *s = AgentState::Waiting);
            let dur = Duration::from_millis(time);
            thread::sleep(dur);
        }
        fn quit(&self) -> bool {
            self.hub
                .control_block
                .as_ref()
                .unwrap()
                .quit
                .load(Ordering::Relaxed)
        }
    }
}

pub trait Behavior {
    fn setup(&mut self) {
        print!("no setup implemented")
    }
    fn done(&mut self) -> bool {
        false
    }
    fn action(&mut self) {
        print!("no action implemented")
    }
    fn failure_detection(&mut self) -> bool {
        true
    }
    fn failure_identification(&mut self) {
        print!("no failure identification implemented")
    }
    fn failure_recovery(&mut self) {
        print!("no failure recovery implemented")
    }
}

pub(crate) fn execute(mut behavior: impl Behavior + TaskControl) {
    behavior.set_thread();
    if behavior.init() == true {
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

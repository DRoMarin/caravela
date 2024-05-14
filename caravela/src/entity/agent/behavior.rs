use crate::entity::agent::Agent;

pub trait Behavior {
    fn agent_builder(base_agent: Agent) -> Self;
    fn agent_mut_ref(&mut self) -> &mut Agent;
    fn setup(&mut self) {
        println!("\n{}: no setup implemented", self.agent_mut_ref().aid());
    }

    fn done(&mut self) -> bool {
        println!(
            "{}: execution done, taking down.\n",
            self.agent_mut_ref().aid()
        );
        true
    }

    fn action(&mut self) {
        println!("{}: no action implemented", self.agent_mut_ref().aid());
    }

    fn failure_detection(&mut self) -> bool {
        println!(
            "{}: no failure detection implemented",
            self.agent_mut_ref().aid()
        );
        false
    }

    fn failure_identification(&mut self) {
        println!(
            "{}: no failure identification implemented",
            self.agent_mut_ref().aid()
        );
    }

    fn failure_recovery(&mut self) {
        println!(
            "{}: no failure recovery implemented",
            self.agent_mut_ref().aid()
        );
    }
}

pub(crate) fn execute(mut behavior: impl Behavior) {
    //behavior.agent_mut_ref().set_thread();
    if behavior.agent_mut_ref().init() {
        behavior.setup();
        loop {
            behavior.agent_mut_ref().suspend();
            if behavior.agent_mut_ref().quit() {
                break;
            }
            behavior.action();
            if behavior.failure_detection() {
                behavior.failure_identification();
                behavior.failure_recovery();
            }
            if behavior.done() {
                behavior.agent_mut_ref().takedown();
                break;
            }
        }
    }
}

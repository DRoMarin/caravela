use crate::entity::agent::Agent;

pub trait Behavior {
    fn agent_builder(base_agent: Agent) -> Self;
    fn get_agent_ref(&mut self) -> &mut Agent;
    fn setup(&mut self) {
        println!(
            "{}: no setup implemented",
            self.get_agent_ref().get_nickname()
        );
    }

    fn done(&mut self) -> bool {
        println!(
            "{}: execution done, taking down...",
            self.get_agent_ref().get_nickname()
        );
        true
    }

    fn action(&mut self) {
        println!(
            "{}: no action implemented",
            self.get_agent_ref().get_nickname()
        );
    }

    fn failure_detection(&mut self) -> bool {
        println!(
            "{}: no failure detection implemented",
            self.get_agent_ref().get_nickname()
        );
        false
    }

    fn failure_identification(&mut self) {
        println!(
            "{}: no failure identification implemented",
            self.get_agent_ref().get_nickname()
        );
    }

    fn failure_recovery(&mut self) {
        println!(
            "{}: no failure recovery implemented",
            self.get_agent_ref().get_nickname()
        );
    }
}

pub(crate) fn execute(mut behavior: impl Behavior) {
    behavior.get_agent_ref().set_thread();
    if behavior.get_agent_ref().init() {
        behavior.setup();
        loop {
            behavior.get_agent_ref().suspend();
            if behavior.get_agent_ref().quit() {
                break;
            }
            behavior.action();
            if behavior.failure_detection() {
                behavior.failure_identification();
                behavior.failure_recovery();
            }
            if behavior.done() {
                behavior.get_agent_ref().takedown();
                break;
            }
        }
    }
}

use crate::entity::agent::Agent;

/// Establishes that an object is an agent.
pub trait Behavior {
    /// Required function to build the derived agent instance.
    fn agent_builder(base_agent: Agent) -> Self;
    /// Required function to access  [`Agent`] base functionality.
    fn agent_mut_ref(&mut self) -> &mut Agent;
    /// Function executed once after starting the agent; just before [`Behavior::action`]. Empty by default.
    fn setup(&mut self) {
        println!(
            "[DEFAULT] {}: no setup implemented",
            self.agent_mut_ref().aid()
        );
    }
    /// Function executed after [`Behavior::action`] used to determined if the agent has reached the end of its life cycle.
    /// Returns `true` by default.
    fn done(&mut self) -> bool {
        println!(
            "[DEFAULT] {}: execution done, taking down",
            self.agent_mut_ref().aid()
        );
        true
    }
    /// Function that corresponds to the main repeating activity of the agent executed after [`Behavior::setup`].
    /// Empty by default.
    fn action(&mut self) {
        println!(
            "[DEFAULT] {}: no action implemented",
            self.agent_mut_ref().aid()
        );
    }
    /// Function used to include Fault Detection as part of the FDIR functionality of the agent.
    /// Returns `false` by default.
    fn failure_detection(&mut self) -> bool {
        println!(
            "[DEFAULT] {}: no failure detection implemented",
            self.agent_mut_ref().aid()
        );
        false
    }
    /// Function used to include Fault Identification as part of the FDIR functionality of the agent.
    /// Empty by default.
    fn failure_identification(&mut self) {
        println!(
            "[DEFAULT] {}: no failure identification implemented",
            self.agent_mut_ref().aid()
        );
    }
    /// Function used to include Fault Recovery as part of the FDIR functionality of the agent.
    /// Empty by default.
    fn failure_recovery(&mut self) {
        println!(
            "[DEFAULT] {}: no failure recovery implemented",
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
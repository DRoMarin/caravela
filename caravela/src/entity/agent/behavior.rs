use crate::entity::agent::{AgentBase, AgentState};
use std::hint;

/// Establishes that an object is an agent.
pub trait Behavior: AgentBase {
    /// Required function to build the derived agent instance.
    //fn agent_builder(base_agent: Agent) -> Self;
    /// Required function to access  [`Agent`] base functionality.
    //fn agent(&mut self) -> &mut Agent;
    /// Function executed once after starting the agent; just before [`Behavior::action`]. Empty by default.
    fn setup(&mut self) {
        caravela_dflt!("{}: no setup implemented", self.agent().aid());
    }
    /// Function executed after [`Behavior::action`] used to determined if the agent has reached the end of its life cycle.
    /// Returns `true` by default.
    fn done(&mut self) -> bool {
        caravela_dflt!("{}: execution done, taking down", self.agent().aid());
        true
    }
    /// Function that corresponds to the main repeating activity of the agent executed after [`Behavior::setup`].
    /// Empty by default.
    fn action(&mut self) {
        caravela_dflt!("{}: no action implemented", self.agent().aid());
    }
    /// Function used to include Fault Detection as part of the FDIR functionality of the agent.
    /// Returns `false` by default.
    fn failure_detection(&mut self) -> bool {
        caravela_dflt!("{}: no failure detection implemented", self.agent().aid());
        false
    }
    /// Function used to include Fault Identification as part of the FDIR functionality of the agent.
    /// Empty by default.
    fn failure_identification(&mut self) {
        caravela_dflt!(
            "{}: no failure identification implemented",
            self.agent().aid()
        );
    }
    /// Function used to include Fault Recovery as part of the FDIR functionality of the agent.
    /// Empty by default.
    fn failure_recovery(&mut self) {
        caravela_dflt!("{}: no failure recovery implemented", self.agent().aid());
    }
}

pub(crate) fn execute(mut behavior: impl Behavior) {
    //behavior.agent_mut_ref().set_thread();
    behavior.agent().init();

    while behavior.agent().tcb.agent_state() == AgentState::Initiated {
        hint::spin_loop()
    }

    behavior.setup();
    loop {
        behavior.agent().suspend();
        if behavior.agent().quit() {
            break;
        }
        behavior.action();
        if behavior.failure_detection() {
            behavior.failure_identification();
            behavior.failure_recovery();
        }
        if behavior.done() {
            break;
        }
    }
    let _ = behavior.agent().takedown();
}

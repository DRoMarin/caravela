use crate::ErrorCode;

use super::Agent;

/// Establishes that an object is an agent.
pub trait Behavior: AsRef<Agent> {
    /// Function executed once after starting the agent; just before [`Behavior::action`]. Empty by default.
    fn setup(&mut self) -> Result<(), ErrorCode> {
        caravela_dflt!("{}: no setup implemented", self.as_ref().name());
        Ok(())
    }
    /// Function executed after [`Behavior::action`] used to determined if the agent has reached the end of its life cycle.
    /// Returns `true` by default.
    fn done(&mut self) -> bool {
        caravela_dflt!("{}: execution done, taking down", self.as_ref().name());
        true
    }
    /// Function that corresponds to the main repeating activity of the agent executed after [`Behavior::setup`].
    /// Empty by default.
    fn action(&mut self) -> Result<(), ErrorCode> {
        caravela_dflt!("{}: no action implemented", self.as_ref().name());
        Ok(())
    }
    /// Function used to include Fault Detection as part of the FDIR functionality of the agent.
    /// Returns `false` by default.
    fn failure_detection(&mut self, action_result: &Result<(), ErrorCode>) -> bool {
        caravela_dflt!(
            "{}: no failure detection implemented ({:?})",
            self.as_ref().name(),
            action_result
        );
        false
    }
    /// Function used to include Fault Identification as part of the FDIR functionality of the agent.
    /// Empty by default.
    fn failure_identification(&mut self, action_result: &Result<(), ErrorCode>) {
        caravela_dflt!(
            "{}: no failure identification implemented ({:?})",
            self.as_ref().name(),
            action_result
        );
    }
    /// Function used to include Fault Recovery as part of the FDIR functionality of the agent.
    /// Empty by default.
    fn failure_recovery(&mut self, action_result: &Result<(), ErrorCode>) {
        caravela_dflt!(
            "{}: no failure recovery implemented ({:?})",
            self.as_ref().name(),
            action_result
        );
    }
}

pub(crate) fn execute(mut behavior: impl Behavior) {
    behavior.as_ref().init();
    let res = behavior.setup();
    if res.is_ok() {
        loop {
            behavior.as_ref().suspend();
            if behavior.as_ref().quit() {
                break;
            }
            let res = behavior.action();
            if behavior.failure_detection(&res) {
                behavior.failure_identification(&res);
                behavior.failure_recovery(&res);
            }
            if behavior.done() {
                let _ = behavior.as_ref().takedown();
                break;
            }
        }
    }
}

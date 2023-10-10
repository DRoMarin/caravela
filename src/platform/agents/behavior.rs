pub trait Behavior {
    fn setup(&mut self);
    fn done(&mut self) -> bool;
    fn action(&mut self);
    fn failure_detection(&mut self) -> bool;
    fn failure_identification(&mut self);
    fn failure_recovery(&mut self);
}

pub fn execute(mut behavior: impl Behavior) {
    behavior.setup();
    loop {
        behavior.action();
        if behavior.failure_detection() {
            behavior.failure_identification();
            behavior.failure_recovery();
        }
        if behavior.done() {
            break;
        }
    }
}

pub trait Behavior {
    fn setup(&mut self){print!("no setup implemented")}
    fn done(&mut self) -> bool {false}
    fn action(&mut self) {print!("no action implemented")}
    fn failure_detection(&mut self) -> bool {true}
    fn failure_identification(&mut self){print!("no failure identification implemented")}
    fn failure_recovery(&mut self){print!("no failure recovery implemented")}
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

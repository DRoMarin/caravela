/*struct ag<T>{
    yo: T
}

pub(in crate) trait gen{fn hola(&self){}
fn otro(){}}
impl<T> gen for ag<T>{
}


trait beha:gen {
    fn adios(&self){}
}

impl<T> beha for ag<T> {
    fn adios(&self) {
        self.hola();
        <ag<T> as gen>::otro();
    }

}

fn nuevo(){
    let a: ag<i32> = ag { yo: 1 };
}
*/

use crate::platform::agent::GenericAgent;

pub trait Behavior: GenericAgent {
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

pub(crate) fn execute(mut behavior: impl Behavior) {
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

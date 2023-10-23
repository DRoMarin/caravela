use self::agent::base::AgentInfoDescription;
use std::thread::ThreadId;

type ID = ThreadId;
//type RX = Receiver<message::Message>;
//type TX = Sender<message::Message>;
type AgentPrio = i32;
type StackSize = usize;

const DEFAULT_STACK: usize = 8;
const MAX_PRIORITY: AgentPrio = 99;
const MAX_SUBSCRIBERS: usize = 64;

pub mod agent;
mod message;
pub mod service;
//pub mod organization;

enum ErrorCode {
    NoError,
    Found,
    HandleNone,
    ListFull,
    Duplicated,
    NotFound,
    Timeout,
    Invalid,
    NotRegistered,
}

#[derive(PartialEq, Eq, Hash)]
pub struct Platform {}
pub(crate) trait Directory<D, I, O> {
    //manage elements
    fn add_element(&mut self, key: I, value:O);
    fn get_element(&self, element: I) -> Option<O>;
    fn remove_element(&mut self, element: I);
    //manage directory
    fn get_directory(&self) -> &D;
    fn clear_directory(&mut self);
    fn refresh_directory(&mut self) {}
}

pub trait GenericAgent {
    //getters
    fn get_aid(&self) -> Option<&AgentInfoDescription>;
    fn get_name(&self) -> &str;
    fn get_priority(&self) -> AgentPrio;
    fn get_stack_size(&self) -> usize;
    //fn get_directory(&self) -> &T;
    //setters
    //fn set_aid(&mut self, aid: AgentInfoDescription);
    //fn set_platform(&mut self, platform: &Platform);
}

trait UserConditions {
    fn registration_condition() -> bool {
        true
    }
    fn deregistration_condition() -> bool {
        true
    }
    fn suspension_condition() -> bool {
        true
    }
    fn resumption_condition() -> bool {
        true
    }
    fn termination_condition() -> bool {
        true
    }
    fn reset_condition() -> bool {
        true
    }
}

/*struct Parent(pub ThreadId);
impl Parent {

}*/

/*
fn hey()->agents::base::AID{
    let x = agents::base::new("yo".to_string(), 1, 20);
    x.get_aid().unwrap()
}
*/

/*Agents have generic data, for implementation the user must:
- create structs for each agent with each behavior data:
struct A {a,b,c}
struct B {d,e,f}
- encapsulate the struct in a user define enum:

enum AgentData{
    AgentA(A)
    AgentB(B)
}

AgentData enum must be passed as type when instantiating agents
*/

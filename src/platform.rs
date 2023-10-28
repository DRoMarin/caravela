use std::{
    sync::mpsc::{Receiver, Sender},
    thread::ThreadId,
};
pub mod agent;
pub mod entity;
pub mod message;
pub mod service;
//pub mod organization;

use entity::dispatcher::Message;

type ID = ThreadId;
type ThreadPriority = i32;
type StackSize = usize;
type TX = Sender<Message>;
type RX = Receiver<Message>;

pub const DEFAULT_STACK: usize = 8;
pub const MAX_PRIORITY: ThreadPriority = 99;
pub const MAX_SUBSCRIBERS: usize = 64;

pub enum ErrorCode {
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
pub struct Platform {
    name: String,
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

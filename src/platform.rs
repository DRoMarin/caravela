use std::thread::ThreadId;

type AID = ThreadId;
//type RX = Receiver<message::Message>;
//type TX = Sender<message::Message>;
type AgentPrio = i32;
type StackSize = usize;

pub mod agent;
pub mod service;
mod message;
//pub mod organization;

enum ErrorCode {
    NoError,
    Found,
    HandleNull,
    ListFull,
    Duplicated,
    NotFound,
    Timeout,
    Invalid,
    NotRegistered,
}

pub (crate) trait Directory<T,U> {
    //manage elements 
    fn add_element(&mut self, element: U);
    fn remove_element(&mut self, element: U);
    //manage directory
    fn get_directory(&self) -> &T;
    fn clear_directory(&mut self);
    fn refresh_directory(&mut self){}
}

pub (crate) trait Generic<T> {
    //getters
    fn get_aid(&self) -> Option<AID>;
    fn get_name(&self) -> &str;
    fn get_platform(&self) -> Option<AID>;
    fn get_priority(&self) -> AgentPrio;
    fn get_stack_size(&self) -> usize;
    fn get_directory(&self) -> &T;
    //setters
    fn set_aid(&mut self, aid: AID);
    fn set_platform(&mut self, platform_aid: AID);
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
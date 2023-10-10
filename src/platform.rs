use std::thread::ThreadId;
use self::agents::base::Generic;

pub mod agents;
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

/*struct Parent(pub ThreadId);
impl Parent {
    
}*/

/* 
fn hey()->agents::base::AID{
    let x = agents::base::new("yo".to_string(), 1, 20);
    x.get_aid().unwrap()
}
*/
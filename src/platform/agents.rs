use std::thread::ThreadId;

pub mod base;
pub mod organization;
pub mod behavior;

const MAX_SUBSCRIBERS: usize = 64;
type AID = ThreadId;
//type RX = Receiver<message::Message>;
//type TX = Sender<message::Message>;
type AgentPrio = i32;
type StackSize = usize;

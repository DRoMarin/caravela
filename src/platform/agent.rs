use std::thread::ThreadId;

pub mod base;
pub mod organization;
pub mod behavior;

const MAX_SUBSCRIBERS: usize = 64;
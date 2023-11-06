use crate::platform::{
    entity::{messaging::Message, Description, ExecutionResources},
    service::{Service, ServiceHub, UserConditions},
    ControlBlockDirectory, Directory,
    {ErrorCode, Platform, DEFAULT_STACK, MAX_PRIORITY, MAX_SUBSCRIBERS},
};
use std::{
    sync::{atomic::Ordering, Arc, Mutex},
    thread::Thread,
};

//AMS Needs a atomic control block for thread lifecycle control
struct AMS<T: UserConditions> {
    //become Service<AMS> or Service<DF>
    service_hub: ServiceHub,
    directory: Arc<Mutex<Directory>>,
    control_block_directory: Arc<Mutex<ControlBlockDirectory>>,
    conditions: T,
}

impl<T: UserConditions> Service<T> for AMS<T> {
    fn new(hap: &Platform, thread: Thread, conditions: T) -> Self {
        let nickname = "AMS".to_string();
        let resources = ExecutionResources::new(MAX_PRIORITY, DEFAULT_STACK);
        let service_hub = ServiceHub::new(nickname.clone(), resources, thread, &hap.name);
        let directory = hap.white_pages.clone();
        let control_block_directory = hap.control_block_directory.clone();
        Self {
            service_hub,
            directory,
            control_block_directory,
            conditions,
        }
    }

    fn service_function(conditions: &impl UserConditions) {}
}

impl<T: UserConditions> AMS<T> {
    pub(crate) fn register_agent(&mut self, nickname: &str, description: Description) -> ErrorCode {
        if !self.conditions.registration_condition() {
            return ErrorCode::Invalid;
        }
        let mut white_pages = self.directory.lock().unwrap();
        if white_pages.capacity().eq(&MAX_SUBSCRIBERS) {
            return ErrorCode::ListFull;
        }
        if white_pages.contains_key(nickname) {
            return ErrorCode::Duplicated;
        }
        white_pages.insert(nickname.to_string(), description);
        let mut tcb = self.control_block_directory.lock().unwrap();
        tcb.get_mut(nickname)
            .unwrap()
            .init
            .store(true, Ordering::Relaxed);
        ErrorCode::NoError
        //set agent as active in dir
    }

    pub(crate) fn deregister_agent(&mut self, nickname: &str) -> ErrorCode {
        if !self.conditions.deregistration_condition() {
            return ErrorCode::Invalid;
        }
        let mut white_pages = self.directory.lock().unwrap();
        if !white_pages.contains_key(nickname) {
            return ErrorCode::NotFound;
        }
        let mut tcb = self.control_block_directory.lock().unwrap();
        tcb.get_mut(nickname)
            .unwrap()
            .quit
            .store(true, Ordering::Relaxed);
        tcb.remove_entry(nickname);
        white_pages.remove_entry(nickname);
        ErrorCode::NoError
    }

    pub(crate) fn terminate_agent(&mut self, nickname: &str) -> ErrorCode {
        if !self.conditions.termination_condition() {
            return ErrorCode::Invalid;
        }
        self.deregister_agent(nickname)
    }

    pub(crate) fn suspend_agent(&self, nickname: &str) -> ErrorCode {
        if !self.conditions.suspension_condition() {
            return ErrorCode::Invalid;
        }
        let white_pages = self.directory.lock().unwrap();
        if !white_pages.contains_key(nickname) {
            return ErrorCode::NotFound;
        }
        let mut tcb = self.control_block_directory.lock().unwrap();
        tcb.get_mut(nickname)
            .unwrap()
            .suspend
            .store(true, Ordering::Relaxed);
        ErrorCode::NoError
        //update agent status
    }

    pub(crate) fn resume_agent(&mut self, nickname: &str) -> ErrorCode {
        if !self.conditions.resumption_condition() {
            return ErrorCode::Invalid;
        }
        let white_pages = self.directory.lock().unwrap();
        if !white_pages.contains_key(nickname) {
            return ErrorCode::NotFound;
        }
        let tcb = self.control_block_directory.lock().unwrap();
        tcb.get(nickname)
            .unwrap()
            .handle
            .as_ref()
            .unwrap()
            .thread()
            .unpark();
        ErrorCode::NoError
        //update agent status
    }

    pub(crate) fn restart_agent(&mut self, nickname: &str) {
        //relaunch agent
    }
}

//fn format_id(name:,platform)
//IDEA
//let a: &std::sync::mpsc::Receiver<Message> = &self.resources.channel.as_ref().unwrap().1;
//let msg = a.recv();

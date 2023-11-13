use crate::platform::{
    entity::{messaging::Message, Description, ExecutionResources},
    service::{Service, ServiceHub, UserConditions},
    ControlBlockDirectory, Directory, HandleDirectory,
    {ErrorCode, Platform, DEFAULT_STACK, MAX_PRIORITY, MAX_SUBSCRIBERS},
};
use std::{
    sync::{atomic::Ordering, Arc, Mutex, RwLock},
    thread::Thread,
};

//AMS Needs a atomic control block for thread lifecycle control
struct AMS<T: UserConditions> {
    //become Service<AMS> or Service<DF>
    service_hub: ServiceHub,
    directory: Arc<RwLock<Directory>>,
    control_block_directory: Arc<RwLock<ControlBlockDirectory>>,
    handle_directory: Arc<Mutex<HandleDirectory>>,
    conditions: T,
}

impl<T: UserConditions> Service for AMS<T> {
    type Conditions = T;
    fn new(hap: &Platform, thread: Thread, conditions: T) -> Self {
        let nickname = "AMS".to_string();
        let resources = ExecutionResources::new(MAX_PRIORITY, DEFAULT_STACK);
        let service_hub = ServiceHub::new(nickname.clone(), resources, thread, &hap.name);
        let directory = hap.white_pages.clone();
        let control_block_directory = hap.control_block_directory.clone();
        let handle_directory = hap.handle_directory.clone();
        Self {
            service_hub,
            directory,
            control_block_directory,
            handle_directory,
            conditions,
        }
    }
    fn search_agent(&self, nickname: &str) -> ErrorCode {
        let white_pages = self.directory.read().unwrap();
        if white_pages.contains_key(nickname) {
            return ErrorCode::Found;
        }
        ErrorCode::NotFound
    }

    fn register_agent(&mut self, nickname: &str, description: Description) -> ErrorCode {
        if !self.conditions.registration_condition() {
            return ErrorCode::Invalid;
        }
        if self.search_agent(nickname) == ErrorCode::Found {
            return ErrorCode::Duplicated;
        }
        let mut white_pages = self.directory.write().unwrap();
        if white_pages.capacity().eq(&MAX_SUBSCRIBERS) {
            return ErrorCode::ListFull;
        }
        white_pages.insert(nickname.to_string(), description);
        let tcb = self.control_block_directory.write().unwrap();
        tcb.get(nickname)
            .unwrap()
            .init
            .store(true, Ordering::Relaxed);
        ErrorCode::NoError
        //set agent as active in dir
    }

    fn deregister_agent(&mut self, nickname: &str) -> ErrorCode {
        if !self.conditions.deregistration_condition() {
            return ErrorCode::Invalid;
        }
        if self.search_agent(nickname) == ErrorCode::NotFound {
            return ErrorCode::NotFound;
        }
        let mut white_pages = self.directory.write().unwrap();
        let mut tcb = self.control_block_directory.write().unwrap();
        tcb.get(nickname)
            .unwrap()
            .quit
            .store(true, Ordering::Relaxed);

        let mut handles = self.handle_directory.lock().unwrap();
        let mut handle = handles.remove(nickname);
        if let Some(handle) = handle.take() {
            let _ = handle.join(); //can add message when Err or Ok
        }
        white_pages.remove_entry(nickname);
        tcb.remove_entry(nickname);
        ErrorCode::NoError
    }

    fn service_function(conditions: Self::Conditions) {
        //TBD
    }
}

impl<T: UserConditions> AMS<T> {
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
        if self.search_agent(nickname) == ErrorCode::Found {
            return ErrorCode::Found;
        }
        let mut tcb = self.control_block_directory.write().unwrap();
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

        if self.search_agent(nickname) == ErrorCode::Found {
            return ErrorCode::NotFound;
        }
        let handles = self.handle_directory.lock().unwrap();
        handles.get(nickname).unwrap().thread().unpark();
        ErrorCode::NoError
        //update agent status
    }

    pub(crate) fn restart_agent(&mut self, nickname: &str) {
        //relaunch agent
    }
}

//CAN REDUCE CODE BY CREATING: CONDITION CHECK VIA ENUM

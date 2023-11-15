use crate::platform::{
    entity::{
        messaging::{Content, MessageType, RequestType},
        Description, Entity, ExecutionResources,
    },
    service::{Service, ServiceHub, UserConditions},
    AgentState, ControlBlockDirectory, HandleDirectory, StateDirectory,
    {ErrorCode, Platform, DEFAULT_STACK, MAX_PRIORITY, MAX_SUBSCRIBERS},
};
use std::sync::{atomic::Ordering, Arc, Mutex, RwLock};

//AMS Needs a atomic control block for thread lifecycle control
pub(crate) struct AMS<T: UserConditions> {
    //become Service<AMS> or Service<DF>
    pub(crate) service_hub: ServiceHub,
    control_block_directory: Arc<RwLock<ControlBlockDirectory>>,
    handle_directory: Arc<Mutex<HandleDirectory>>,
    state_directory: Arc<RwLock<StateDirectory>>,
    conditions: T,
}

impl<T: UserConditions> Service for AMS<T> {
    type Conditions = T;
    fn new(hap: &Platform, conditions: T) -> Self {
        let nickname = "AMS".to_string();
        let resources = ExecutionResources::new(MAX_PRIORITY, DEFAULT_STACK);
        let directory = hap.white_pages.clone();
        let service_hub = ServiceHub::new(nickname.clone(), resources, &hap.name, directory);
        let control_block_directory = hap.control_block_directory.clone();
        let handle_directory = hap.handle_directory.clone();
        let state_directory = hap.state_directory.clone();
        Self {
            service_hub,
            control_block_directory,
            handle_directory,
            state_directory,
            conditions,
        }
    }
    fn search_agent(&self, nickname: &str) -> ErrorCode {
        let white_pages = self.service_hub.directory.read().unwrap();
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
        let mut white_pages = self.service_hub.directory.write().unwrap();
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
        let mut white_pages = self.service_hub.directory.write().unwrap();
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
    fn service_function(&mut self) {
        self.service_hub.aid.set_thread();
        loop {
            println!("\nwaiting...\n");
            let msg_type = self.service_hub.receive();
            if msg_type != MessageType::Request {
                self.service_hub.msg.set_type(MessageType::NotUnderstood);
            } else if let Content::Request(x) = self.service_hub.msg.get_content().unwrap() {
                let error = match x {
                    RequestType::Register(nickname, description) => {
                        self.register_agent(&nickname, description)
                    }
                    RequestType::Deregister(nickname) => self.deregister_agent(&nickname),
                    RequestType::Suspend(nickname) => self.suspend_agent(&nickname),
                    RequestType::Resume(nickname) => self.resume_agent(&nickname),
                    RequestType::Terminate(nickname) => self.terminate_agent(&nickname),
                    RequestType::Search(nickname) => self.search_agent(&nickname),
                };
                if error == ErrorCode::NoError || error == ErrorCode::Found {
                    self.service_hub.msg.set_type(MessageType::Confirm);
                } else {
                    self.service_hub.msg.set_type(MessageType::Refuse);
                }
            }
            let sender = self.service_hub.msg.get_sender().unwrap().get_name();
            self.service_hub.send_to(&sender);
        }
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
        {
            if self
                .state_directory
                .read()
                .unwrap()
                .get(nickname)
                .unwrap()
                .clone()
                != AgentState::Active
            {
                return ErrorCode::Invalid;
            }
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
        {
            if self
                .state_directory
                .read()
                .unwrap()
                .get(nickname)
                .unwrap()
                .clone()
                != AgentState::Suspended
            {
                return ErrorCode::Invalid;
            }
        }
        if self.search_agent(nickname) == ErrorCode::Found {
            return ErrorCode::NotFound;
        }
        let handles = self.handle_directory.lock().unwrap();
        handles.get(nickname).unwrap().thread().unpark();
        ErrorCode::NoError
        //update agent status
    }

    /*  pub(crate) fn restart_agent(&mut self, nickname: &str) {
            //relaunch agent
        }
    */
}

//CAN REDUCE CODE BY CREATING: CONDITION CHECK VIA ENUM

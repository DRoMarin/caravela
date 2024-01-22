use crate::platform::{
    entity::{
        messaging::{Content, MessageType, RequestType},
        Description, Entity, ExecutionResources,
    },
    service::{Service, ServiceHub, UserConditions},
    AgentState, PrivateRecord, SharedRecord,
    {ErrorCode, Platform, DEFAULT_STACK, MAX_PRIORITY, MAX_SUBSCRIBERS},
};
use std::sync::{atomic::Ordering, mpsc::TrySendError, Arc, RwLock};

//AMS Needs a atomic control block for thread lifecycle control
pub(crate) struct AMS<T: UserConditions> {
    //become Service<AMS> or Service<DF>
    pub(crate) service_hub: ServiceHub,
    shared_platform: Arc<RwLock<SharedRecord>>,
    private_platform: Arc<RwLock<PrivateRecord>>,
    conditions: T,
}

impl<T: UserConditions> Entity for AMS<T> {
    fn get_aid(&self) -> Description {
        self.service_hub.aid.clone()
    }
    fn get_nickname(&self) -> String {
        self.service_hub.nickname.clone()
    }
    fn get_hap(&self) -> String {
        self.service_hub.hap.clone()
    }
    fn get_resources(&self) -> ExecutionResources {
        self.service_hub.resources.clone()
    }
    fn send_to(&mut self, agent: &str) -> ErrorCode {
        let receiver = match self
            .private_platform
            .read()
            .unwrap()
            .white_pages_directory
            .get(agent)
        {
            Some(x) => x.clone(),
            None => return ErrorCode::NotRegistered,
        };
        self.send_to_aid(receiver)
    }
    fn send_to_aid(&mut self, description: Description) -> ErrorCode {
        let address = description.get_address();
        self.service_hub
            .msg
            .set_sender(self.service_hub.aid.clone());
        self.service_hub.msg.set_receiver(description);
        let result = address.try_send(self.service_hub.msg.clone());
        let error_code = match result {
            Ok(_) => ErrorCode::NoError,
            Err(error) => match error {
                TrySendError::Full(_) => ErrorCode::Timeout,
                TrySendError::Disconnected(_) => ErrorCode::NotRegistered, //LIST MAY BE OUTDATED
            },
        };
        error_code
    }
    fn receive(&mut self) -> MessageType {
        let result = self.service_hub.rx.recv();
        let msg_type = match result {
            Ok(received_msg) => {
                self.service_hub.msg = received_msg;
                self.service_hub.msg.get_type().clone().unwrap()
            }
            Err(_) => MessageType::NoResponse,
        }; //could handle Err incase of disconnection
        msg_type
    }
}

impl<T: UserConditions> Service for AMS<T> {
    type Conditions = T;
    fn new(hap: &Platform, conditions: T) -> Self {
        let nickname = "AMS".to_string();
        let resources = ExecutionResources::new(MAX_PRIORITY, DEFAULT_STACK);
        let shared_platform = hap.shared_record.clone();
        let private_platform = hap.private_record.clone();
        let service_hub = ServiceHub::new(
            nickname.clone(),
            resources,
            &shared_platform.read().unwrap().name,
        );
        Self {
            service_hub,
            private_platform,
            shared_platform,
            conditions,
        }
    }
    fn search_agent(&self, nickname: &str) -> ErrorCode {
        let white_pages = &self.private_platform.read().unwrap().white_pages_directory;
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
        let platform = &mut self.private_platform.write().unwrap();
        if platform.white_pages_directory.len().eq(&MAX_SUBSCRIBERS) {
            return ErrorCode::ListFull;
        }
        platform
            .white_pages_directory
            .insert(nickname.to_string(), description);
        platform
            .control_block_directory
            .get(nickname)
            .unwrap()
            .init
            .wait();

        println!("SUCCESSFULLY REGISTERED {}", nickname);
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
        let platform = &mut self.private_platform.write().unwrap();
        let mut handle = platform.handle_directory.remove(nickname);
        if let Some(handle) = handle.take() {
            let _ = handle.join(); //can add message when Err or Ok
        }
        platform.white_pages_directory.remove_entry(nickname);
        platform.control_block_directory.remove_entry(nickname);

        println!(
            "{}: SUCCESSFULLY DEREGISTERED {}",
            self.get_nickname(),
            nickname
        );
        ErrorCode::NoError
    }
    fn service_function(&mut self) {
        self.service_hub.aid.set_thread();
        loop {
            println!("{}: waiting...", self.get_nickname());
            let msg_type = self.receive();
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
                    RequestType::Search(nickname) => {
                        let result = self.search_agent(&nickname);
                        if result == ErrorCode::Found {
                            let found = self
                                .private_platform
                                .read()
                                .unwrap()
                                .white_pages_directory
                                .get(&nickname)
                                .unwrap()
                                .clone();
                            self.service_hub.msg.set_type(MessageType::Inform);
                            self.service_hub.msg.set_content(Content::AID(found));
                        } else {
                            self.service_hub.msg.set_type(MessageType::Failure);
                            self.service_hub.msg.set_content(Content::None);
                        }
                        let receiver = self.service_hub.msg.get_sender().unwrap();
                        self.send_to_aid(receiver);
                        result
                    }
                    RequestType::None => ErrorCode::Invalid,
                };
                if error == ErrorCode::NoError || error == ErrorCode::Found {
                    self.service_hub.msg.set_type(MessageType::Confirm);
                } else {
                    self.service_hub.msg.set_type(MessageType::Refuse);
                }
            }
            let sender = self.service_hub.msg.get_sender().unwrap().get_name();
            self.send_to(&sender);
        }
    }
}

impl<T: UserConditions> AMS<T> {
    pub(crate) fn terminate_agent(&mut self, nickname: &str) -> ErrorCode {
        if !self.conditions.termination_condition() {
            return ErrorCode::Invalid;
        }
        {
            let platform = &mut self.private_platform.write().unwrap();
            platform
                .control_block_directory
                .get(nickname)
                .unwrap()
                .quit
                .store(true, Ordering::Relaxed);
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
            if *self
                .shared_platform
                .read()
                .unwrap()
                .state_directory
                .get(nickname)
                .unwrap()
                != AgentState::Active
            {
                return ErrorCode::Invalid;
            }
        }
        let platform = &mut self.private_platform.write().unwrap();
        platform
            .control_block_directory
            .get(nickname)
            .unwrap()
            .suspend
            .store(true, Ordering::Relaxed);

        ErrorCode::NoError
    }
    pub(crate) fn resume_agent(&mut self, nickname: &str) -> ErrorCode {
        if !self.conditions.resumption_condition() {
            return ErrorCode::Invalid;
        }
        {
            if *self
                .shared_platform
                .read()
                .unwrap()
                .state_directory
                .get(nickname)
                .unwrap()
                != AgentState::Suspended
            {
                return ErrorCode::Invalid;
            }
        }
        if self.search_agent(nickname) == ErrorCode::Found {
            return ErrorCode::NotFound;
        }
        let handles = &mut self.private_platform.write().unwrap().handle_directory;
        handles.get(nickname).unwrap().thread().unpark();
        ErrorCode::NoError
    }

    /*  pub(crate) fn restart_agent(&mut self, nickname: &str) {
            //relaunch agent
        }
    */
    /*  pub(crate) fn modify_agent(&mut self, nickname: &str, update: Description) {
            //update agent
        }
    */
}

//CAN REDUCE CODE BY CREATING: CONDITION CHECK VIA ENUM

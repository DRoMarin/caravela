use crate::platform::{
    deck::{Deck, SyncType, TcbField},
    entity::{
        messaging::{Content, MessageType, RequestType},
        Description, Entity, ExecutionResources,
    },
    service::{Service, ServiceHub, UserConditions},
    AgentState, ErrorCode, Platform, DEFAULT_STACK, MAX_PRIORITY,
};
use std::sync::{Arc, RwLock};

//AMS Needs a atomic control block for thread lifecycle control
pub(crate) struct AMS<T: UserConditions> {
    //become Service<AMS> or Service<DF>
    pub(crate) service_hub: ServiceHub,
    deck: Arc<RwLock<Deck>>,
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
        self.deck.read().unwrap().send(
            self.get_aid(),
            agent,
            self.service_hub.msg.clone(),
            SyncType::Blocking,
        )
    }
    fn send_to_aid(&mut self, description: Description) -> ErrorCode {
        self.deck.read().unwrap().send_to_aid(
            self.get_aid(),
            description,
            self.service_hub.msg.clone(),
            SyncType::Blocking,
        )
    }
    fn receive(&mut self) -> MessageType {
        let result = self.service_hub.rx.recv();
        let msg_type = match result {
            Ok(received_msg) => {
                self.service_hub.msg = received_msg;
                self.service_hub.msg.get_type().unwrap()
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
        let platform = &hap.name;
        let deck = hap.deck.clone();
        let service_hub = ServiceHub::new(nickname.clone(), resources, platform);
        Self {
            service_hub,
            deck,
            conditions,
        }
    }
    fn search_agent(&self, nickname: &str) -> ErrorCode {
        self.deck.read().unwrap().search_agent(nickname) //ADD ARGS
    }
    fn register_agent(&mut self, nickname: &str) -> ErrorCode {
        if !self.conditions.registration_condition() {
            return ErrorCode::Invalid;
        }
        if self.search_agent(nickname) == ErrorCode::Found {
            return ErrorCode::Duplicated;
        }
        let description = self.service_hub.msg.get_sender().unwrap();
        self.deck
            .write()
            .unwrap()
            .insert_agent(nickname, description)
    }
    fn deregister_agent(&mut self, nickname: &str) -> ErrorCode {
        if !self.conditions.deregistration_condition() {
            return ErrorCode::Invalid;
        }
        if self.search_agent(nickname) == ErrorCode::NotFound {
            return ErrorCode::NotFound;
        }
        /*
        println!(
            "{}: SUCCESSFULLY DEREGISTERED {}",
            self.get_nickname(),
            nickname
        );
        */
        self.deck.write().unwrap().remove_agent(nickname)
    }
    fn service_function(&mut self) {
        self.service_hub.aid.set_thread();
        loop {
            println!("{}: WAITING...", self.get_nickname());
            let msg_type = self.receive();
            if msg_type != MessageType::Request {
                self.service_hub.msg.set_type(MessageType::NotUnderstood);
                self.service_hub.msg.set_content(Content::None);
            } else if let Content::Request(request_type) =
                self.service_hub.msg.get_content().unwrap()
            {
                let error = match request_type.clone() {
                    RequestType::Register(nickname) => self.register_agent(&nickname),
                    RequestType::Deregister(nickname) => self.deregister_agent(&nickname),
                    RequestType::Suspend(nickname) => self.suspend_agent(&nickname),
                    RequestType::Resume(nickname) => self.resume_agent(&nickname),
                    RequestType::Terminate(nickname) => self.terminate_agent(&nickname),
                    RequestType::Search(nickname) => self.search_agent(&nickname),
                    _ => ErrorCode::InvalidRequest,
                };
                self.service_request_reply_type(request_type, error);
            }
            // setting up reply
            let receiver = self.service_hub.msg.get_sender().unwrap();
            self.service_hub.msg.set_sender(self.get_aid());
            println!(
                "{}: REPLYING TO {}",
                self.get_nickname(),
                receiver.get_name()
            );
            self.send_to_aid(receiver);
            /*println!(
                "{}",
                self.private_platform.read().unwrap().handle_directory.len()
            );*/
            /*
            if self
                .private_platform
                .read()
                .unwrap()
                .handle_directory
                .len()
                .eq(&1)
            {
                println!("{}: CLOSING PLATFORM", self.get_nickname());
                break;
            }*/
        }
    }
    /*  pub(crate) fn modify_agent(&mut self, nickname: &str, update: Description) {
            //update agent
        }
    */
    fn service_request_reply_type(&mut self, request_type: RequestType, error: ErrorCode) {
        if let RequestType::Search(nickname) = request_type {
            if error == ErrorCode::Found {
                let found = self.deck.read().unwrap().get_agent(&nickname);
                self.service_hub.msg.set_content(Content::AID(found));
                self.service_hub.msg.set_type(MessageType::Inform);
            } else {
                self.service_hub.msg.set_content(Content::None);
                self.service_hub.msg.set_type(MessageType::Failure);
            }
        } else {
            self.service_hub.msg.set_content(Content::None);
            self.service_hub
                .msg
                .set_type(<AMS<T> as Service>::error_to_msgtype(error))
        }
    }
}

impl<T: UserConditions> AMS<T> {
    pub(crate) fn terminate_agent(&mut self, nickname: &str) -> ErrorCode {
        if !self.conditions.termination_condition() {
            return ErrorCode::Invalid;
        }
        if self.search_agent(nickname) == ErrorCode::NotFound {
            return ErrorCode::NotFound;
        }
        let mut deck_guard = self.deck.write().unwrap();
        let state = deck_guard.get_agent_state(nickname);
        if state != AgentState::Active {
            return ErrorCode::Invalid;
        }
        deck_guard.modify_control_block(nickname, TcbField::Quit, true);
        deck_guard.remove_agent(nickname)
    }
    pub(crate) fn suspend_agent(&self, nickname: &str) -> ErrorCode {
        if !self.conditions.suspension_condition() {
            return ErrorCode::Invalid;
        }
        if self.search_agent(nickname) == ErrorCode::NotFound {
            return ErrorCode::NotFound;
        }
        let mut deck_guard = self.deck.write().unwrap();
        let state = deck_guard.get_agent_state(nickname);
        if state != AgentState::Active {
            return ErrorCode::Invalid;
        }
        deck_guard.modify_control_block(nickname, TcbField::Suspend, true);
        ErrorCode::NoError
    }
    pub(crate) fn resume_agent(&mut self, nickname: &str) -> ErrorCode {
        if !self.conditions.resumption_condition() {
            return ErrorCode::Invalid;
        }
        if self.search_agent(nickname) == ErrorCode::NotFound {
            return ErrorCode::NotFound;
        }
        let mut deck_guard = self.deck.write().unwrap();
        let state = deck_guard.get_agent_state(nickname);
        if state != AgentState::Suspended {
            return ErrorCode::Invalid;
        }
        deck_guard.modify_control_block(nickname, TcbField::Suspend, false);
        self.deck.write().unwrap().unpark_agent(nickname);
        ErrorCode::NoError
    }

    /*  pub(crate) fn restart_agent(&mut self, nickname: &str) {
            //relaunch agent
        }
    */
}

//CAN REDUCE CODE BY CREATING: CONDITION CHECK VIA ENUM

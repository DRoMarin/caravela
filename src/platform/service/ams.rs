use crate::platform::{
    deck::{Deck, SyncType, TcbField},
    entity::{
        messaging::{Content, MessageType, RequestType},
        Description, Entity, ExecutionResources,
    },
    service::{Service, ServiceHub, UserConditions},
    AgentState, ErrorCode, Platform, DEFAULT_STACK, MAX_PRIORITY,
};
use std::sync::{mpsc::RecvError, Arc, RwLock};

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

    fn send_to(&mut self, agent: &str) -> Result<(), ErrorCode> {
        self.service_hub.msg.set_sender(self.get_aid());
        self.deck.read().unwrap().send(
            agent,
            self.service_hub.msg.clone(),
            SyncType::Blocking,
        )
    }

    fn send_to_aid(&mut self, description: Description) -> Result<(), ErrorCode> {
        self.service_hub.msg.set_sender(self.get_aid());
        self.deck.read().unwrap().send_to_aid(
            description,
            self.service_hub.msg.clone(),
            SyncType::Blocking,
        )
    }

    fn receive(&mut self) -> Result<MessageType, RecvError> {
        let result = self.service_hub.rx.recv();
        match result {
            Ok(received_msg) => {
                self.service_hub.msg = received_msg;
                println!(
                    "SENDER: {}",
                    self.service_hub.msg.get_sender().unwrap().get_name()
                );
                Ok(self.service_hub.msg.get_type().unwrap())
            }
            Err(err) => Err(err),
        } //could handle Err incase of disconnection
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

    fn search_agent(&self, nickname: &str) -> Result<(), ErrorCode> {
        self.deck.read().unwrap().search_agent(nickname) //ADD ARGS
    }

    fn register_agent(&mut self, nickname: &str) -> Result<(), ErrorCode> {
        if !self.conditions.registration_condition() {
            return Err(ErrorCode::Invalid);
        }
        if let Err(_) = self.search_agent(nickname) {
            return Err(ErrorCode::Duplicated);
        }
        let description = self.service_hub.msg.get_sender().unwrap();
        self.deck
            .write()
            .unwrap()
            .insert_agent(nickname, description)
    }

    fn deregister_agent(&mut self, nickname: &str) -> Result<(), ErrorCode> {
        if !self.conditions.deregistration_condition() {
            return Err(ErrorCode::Invalid);
        }
        if let Err(_) = self.search_agent(nickname) {
            return Err(ErrorCode::NotFound);
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
            let msg_result = self.receive();
            if Ok(MessageType::Request) == msg_result {
                if let Content::Request(request_type) = self.service_hub.msg.get_content().unwrap()
                {
                    let error = match request_type.clone() {
                        RequestType::Register(nickname) => self.register_agent(&nickname),
                        RequestType::Deregister(nickname) => self.deregister_agent(&nickname),
                        RequestType::Suspend(nickname) => self.suspend_agent(&nickname),
                        RequestType::Resume(nickname) => self.resume_agent(&nickname),
                        RequestType::Terminate(nickname) => self.terminate_agent(&nickname),
                        RequestType::Search(nickname) => self.search_agent(&nickname),
                        _ => Err(ErrorCode::InvalidRequest),
                    };
                    self.service_req_reply_type(request_type, error);
                } else {
                    self.service_hub.msg.set_type(MessageType::NotUnderstood);
                    self.service_hub.msg.set_content(Content::None);
                }
                // setting up reply
                let receiver = self.service_hub.msg.get_sender().unwrap();
                self.service_hub.msg.set_sender(self.get_aid());
                println!(
                    "{}: REPLYING TO {}",
                    self.get_nickname(),
                    receiver.get_name()
                );
                let _ = self.send_to_aid(receiver);
            }
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

    fn service_req_reply_type(&mut self, request_type: RequestType, result: Result<(), ErrorCode>) {
        self.service_hub.msg.set_content(Content::None);
        if let Err(_error) = result {
            self.service_hub.msg.set_type(MessageType::Failure);
        } else {
            self.service_hub.msg.set_type(MessageType::Inform);
            if let RequestType::Search(nickname) = request_type {
                let found = self.deck.read().unwrap().get_agent(&nickname);
                self.service_hub.msg.set_content(Content::AID(found));
            }
        }
    }
}

impl<T: UserConditions> AMS<T> {
    pub(crate) fn terminate_agent(&mut self, nickname: &str) -> Result<(), ErrorCode> {
        if !self.conditions.termination_condition() {
            return Err(ErrorCode::Invalid);
        }
        if let Err(_) = self.search_agent(nickname) {
            return Err(ErrorCode::NotFound);
        }
        let mut deck_guard = self.deck.write().unwrap();
        let state = deck_guard.get_agent_state(nickname);
        if state != AgentState::Active {
            return Err(ErrorCode::Invalid);
        }
        deck_guard.modify_control_block(nickname, TcbField::Quit, true);
        deck_guard.remove_agent(nickname)
    }

    pub(crate) fn suspend_agent(&self, nickname: &str) -> Result<(), ErrorCode> {
        if !self.conditions.suspension_condition() {
            return Err(ErrorCode::Invalid);
        }
        if let Err(_) = self.search_agent(nickname) {
            return Err(ErrorCode::NotFound);
        }
        let mut deck_guard = self.deck.write().unwrap();
        let state = deck_guard.get_agent_state(nickname);
        if state != AgentState::Active {
            return Err(ErrorCode::Invalid);
        }
        deck_guard.modify_control_block(nickname, TcbField::Suspend, true);
        Ok(())
    }

    pub(crate) fn resume_agent(&mut self, nickname: &str) -> Result<(), ErrorCode> {
        if !self.conditions.resumption_condition() {
            return Err(ErrorCode::Invalid);
        }
        if let Err(_) = self.search_agent(nickname) {
            return Err(ErrorCode::NotFound);
        }
        let mut deck_guard = self.deck.write().unwrap();
        let state = deck_guard.get_agent_state(nickname);
        if state != AgentState::Suspended {
            return Err(ErrorCode::Invalid);
        }
        deck_guard.modify_control_block(nickname, TcbField::Suspend, false);
        self.deck.write().unwrap().unpark_agent(nickname);
        Ok(())
    }

    /*  pub(crate) fn restart_agent(&mut self, nickname: &str) {
            //relaunch agent
        }
    */
}

//CAN REDUCE CODE BY CREATING: CONDITION CHECK VIA ENUM

use crate::platform::{
    deck::TcbField,
    entity::{
        messaging::{Content, MessageType, RequestType},
        ExecutionResources, Hub,
    },
    service::{Service, UserConditions},
    AgentState, ErrorCode, Platform, DEFAULT_STACK, MAX_PRIORITY,
};

//AMS Needs a atomic control block for thread lifecycle control
pub(crate) struct AMS<T: UserConditions> {
    //become Service<AMS> or Service<DF>
    pub(crate) hub: Hub,
    //pub msg: Message,
    conditions: T,
}

impl<T: UserConditions> Service for AMS<T> {
    type Conditions = T;
    fn new(hap: &Platform, conditions: T) -> Self {
        let nickname = "AMS".to_string();
        let resources = ExecutionResources::new(MAX_PRIORITY, DEFAULT_STACK);
        let platform = hap.name.clone();
        let deck = hap.deck.clone();
        let hub = Hub::new(nickname.clone(), resources, deck, platform);
        Self { hub, conditions }
    }

    fn search_agent(&self, nickname: &str) -> Result<(), ErrorCode> {
        self.hub.deck.read().unwrap().search_agent(nickname) //ADD ARGS
    }

    fn register_agent(&mut self, nickname: &str) -> Result<(), ErrorCode> {
        if !self.conditions.registration_condition() {
            return Err(ErrorCode::Invalid);
        }
        if let Err(_) = self.search_agent(nickname) {
            return Err(ErrorCode::Duplicated);
        }
        let description = self.hub.get_msg().get_sender().unwrap();
        self.hub
            .deck
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
        self.hub.deck.write().unwrap().remove_agent(nickname)
    }

    fn service_function(&mut self) {
        self.hub.aid.set_thread();
        loop {
            println!("{}: WAITING...", self.hub.get_nickname());
            let msg_result = self.hub.receive();
            let receiver = self.hub.get_msg().get_sender().unwrap();
            if Ok(MessageType::Request) == msg_result {
                if let Content::Request(request_type) = self.hub.get_msg().get_content() {
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
                    self.hub.set_msg(MessageType::NotUnderstood, Content::None);
                }
                // setting up reply
                println!(
                    "{}: REPLYING TO {}",
                    self.hub.get_nickname(),
                    receiver.get_name()
                );
                let _ = self.hub.send_to_aid(receiver);
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
        match result {
            Ok(()) => {
                let msg_content = if let RequestType::Search(nickname) = request_type {
                    Content::AID(self.hub.deck.read().unwrap().get_agent(&nickname))
                } else {
                    Content::None
                };
                self.hub.set_msg(MessageType::Inform, msg_content);
            }
            Err(_) => self.hub.set_msg(MessageType::Failure, Content::None),
        };
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
        let mut deck_guard = self.hub.deck.write().unwrap();
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
        let mut deck_guard = self.hub.deck.write().unwrap();
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
        let mut deck_guard = self.hub.deck.write().unwrap();
        let state = deck_guard.get_agent_state(nickname);
        if state != AgentState::Suspended {
            return Err(ErrorCode::Invalid);
        }
        deck_guard.modify_control_block(nickname, TcbField::Suspend, false);
        self.hub.deck.write().unwrap().unpark_agent(nickname);
        Ok(())
    }

    /*  pub(crate) fn restart_agent(&mut self, nickname: &str) {
            //relaunch agent
        }
    */
}

//CAN REDUCE CODE BY CREATING: CONDITION CHECK VIA ENUM

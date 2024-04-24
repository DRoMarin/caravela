use std::sync::{Arc, RwLock};

use crate::{
    deck::{Deck, TcbField},
    entity::{
        agent::AgentState,
        messaging::{Content, MessageType, RequestType},
        service::{Service, UserConditions},
        ExecutionResources, Hub,
    },
    ErrorCode, DEFAULT_STACK, MAX_PRIORITY,
};

//AMS Needs a atomic control block for thread lifecycle control
#[derive(Debug)]
pub(crate) struct Ams<T: UserConditions> {
    //become Service<AMS> or Service<DF>
    pub(crate) hub: Hub,
    //pub msg: Message,
    conditions: T,
}

impl<T: UserConditions> Service for Ams<T> {
    type Conditions = T;
    fn new(hap: String, deck: Arc<RwLock<Deck>>, conditions: T) -> Self {
        let name = "AMS".to_string();
        let resources = ExecutionResources::new(MAX_PRIORITY, DEFAULT_STACK);
        //let platform = hap.name.clone();
        //let platform = hap.name.clone();
        //let deck = hap.deck.clone();
        //let hub = Hub::new(name.clone(), resources, deck, platform);
        let hub = Hub::new(name, resources, deck, hap);
        Self { hub, conditions }
    }

    fn search_agent(&self, name: &str) -> Result<(), ErrorCode> {
        self.hub.arc_deck().read().unwrap().search_agent(name) //ADD ARGS
    }

    fn register_agent(&mut self, name: &str) -> Result<(), ErrorCode> {
        if !self.conditions.registration_condition() {
            return Err(ErrorCode::Invalid);
        }
        if self.search_agent(name).is_err() {
            return Err(ErrorCode::Duplicated);
        }
        let description = self.hub.msg().sender().unwrap();
        self.hub
            .arc_deck()
            .write()
            .unwrap()
            .insert_agent(name, description)
    }

    fn deregister_agent(&mut self, name: &str) -> Result<(), ErrorCode> {
        if !self.conditions.deregistration_condition() {
            return Err(ErrorCode::Invalid);
        }
        if self.search_agent(name).is_err() {
            return Err(ErrorCode::NotFound);
        }
        /*
        println!(
            "{}: SUCCESSFULLY DEREGISTERED {}",
            self.get_nickname(),
            nickname
        );
        */
        self.hub.arc_deck().write().unwrap().remove_agent(name)
    }

    fn service_function(&mut self) {
        self.hub.set_thread();
        loop {
            //println!("{}: WAITING...", self.hub.get_nickname());
            println!("{}: WAITING...", self.hub.aid());
            let msg_result = self.hub.receive();
            let receiver = self.hub.msg().sender().unwrap();
            if Ok(MessageType::Request) == msg_result {
                if let Content::Request(request_type) = self.hub.msg().content() {
                    let error = match request_type.clone() {
                        RequestType::Register(name) => self.register_agent(&name),
                        RequestType::Deregister(name) => self.deregister_agent(&name),
                        RequestType::Suspend(name) => self.suspend_agent(&name),
                        RequestType::Resume(name) => self.resume_agent(&name),
                        RequestType::Terminate(name) => self.terminate_agent(&name),
                        RequestType::Search(name) => self.search_agent(&name),
                        _ => Err(ErrorCode::InvalidRequest),
                    };
                    self.service_req_reply_type(request_type, error);
                } else {
                    self.hub.set_msg(MessageType::NotUnderstood, Content::None);
                }
                // setting up reply
                println!(
                    "{}: REPLYING TO {}",
                    //self.hub.get_nickname(),
                    self.hub.aid(),
                    receiver
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
                let msg_content = if let RequestType::Search(name) = request_type {
                    Content::AID(self.hub.arc_deck().read().unwrap().agent_aid(&name))
                } else {
                    Content::None
                };
                self.hub.set_msg(MessageType::Inform, msg_content);
            }
            Err(_) => self.hub.set_msg(MessageType::Failure, Content::None),
        };
    }
}

impl<T: UserConditions> Ams<T> {
    pub(crate) fn terminate_agent(&mut self, name: &str) -> Result<(), ErrorCode> {
        if !self.conditions.termination_condition() {
            return Err(ErrorCode::Invalid);
        }
        if self.search_agent(name).is_err() {
            return Err(ErrorCode::NotFound);
        }
        let arc_deck = self.hub.arc_deck();
        let mut deck_guard = arc_deck.write().unwrap();
        let state = deck_guard.agent_state(name);
        if state != AgentState::Active {
            return Err(ErrorCode::Invalid);
        }
        deck_guard.modify_control_block(name, TcbField::Quit, true);
        deck_guard.remove_agent(name)
    }

    pub(crate) fn suspend_agent(&self, name: &str) -> Result<(), ErrorCode> {
        if !self.conditions.suspension_condition() {
            return Err(ErrorCode::Invalid);
        }
        if self.search_agent(name).is_err() {
            return Err(ErrorCode::NotFound);
        }
        let arc_deck = self.hub.arc_deck();
        let mut deck_guard = arc_deck.write().unwrap();
        let state = deck_guard.agent_state(name);
        if state != AgentState::Active {
            return Err(ErrorCode::Invalid);
        }
        deck_guard.modify_control_block(name, TcbField::Suspend, true);
        Ok(())
    }

    pub(crate) fn resume_agent(&mut self, name: &str) -> Result<(), ErrorCode> {
        if !self.conditions.resumption_condition() {
            return Err(ErrorCode::Invalid);
        }
        if self.search_agent(name).is_err() {
            return Err(ErrorCode::NotFound);
        }
        let arc_deck = self.hub.arc_deck();
        let mut deck_guard = arc_deck.write().unwrap();
        let state = deck_guard.agent_state(name);
        if state != AgentState::Suspended {
            return Err(ErrorCode::Invalid);
        }
        deck_guard.modify_control_block(name, TcbField::Suspend, false);
        self.hub.arc_deck().write().unwrap().unpark_agent(name);
        Ok(())
    }

    /*  pub(crate) fn restart_agent(&mut self, nickname: &str) {
            //relaunch agent
        }
    */
}

//CAN REDUCE CODE BY CREATING: CONDITION CHECK VIA ENUM

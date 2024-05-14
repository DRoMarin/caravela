use crate::{
    deck::{Deck, TcbField},
    entity::{
        agent::AgentState,
        messaging::{Content, MessageType, RequestType},
        service::{Service, UserConditions},
        Hub,
    },
    Description, ErrorCode, RX,
};
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug)]
pub struct AmsAgentDescription {
    aid: Description,
}

impl AmsAgentDescription {
    pub fn new(aid: Description) -> Self {
        Self { aid }
    }

    pub fn aid(&self) -> &Description {
        &self.aid
    }
}

//AMS Needs an atomic control block for thread lifecycle control
#[derive(Debug)]
pub(crate) struct Ams<T: UserConditions> {
    //become Service<AMS> or Service<DF>
    pub(crate) hub: Hub,
    conditions: T,
}

impl<T: UserConditions> Service for Ams<T> {
    type Conditions = T;
    fn new(rx: RX, deck: Arc<RwLock<Deck>>, conditions: Self::Conditions) -> Self {
        let hub = Hub::new(rx, deck);
        Self { hub, conditions }
    }

    fn init(&mut self) {
        todo!();
    }

    fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        self.hub.deck_read()?.search_agent(aid)?;
        Ok(())
    }
    fn register_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        if !self.conditions.registration_condition() {
            return Err(ErrorCode::InvalidConditions(RequestType::Register(
                aid.clone(),
            )));
        }
        if self.search_agent(aid).is_err() {
            return Err(ErrorCode::Duplicated);
        }
        let description = self.hub.msg().sender().to_owned();
        self.hub.deck_write()?.insert_agent(description)
    }

    fn deregister_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        if !self.conditions.deregistration_condition() {
            return Err(ErrorCode::InvalidConditions(RequestType::Deregister(
                aid.clone(),
            )));
        }
        self.hub.deck_write()?.remove_agent(aid)
    }

    fn service_function(&mut self) {
        //self.hub.set_thread();
        loop {
            println!("{}: WAITING...", self.hub.aid());
            let msg_result = self.hub.receive();
            dbg!(self.hub.msg());
            let receiver = self.hub.msg().sender().clone();
            if Ok(MessageType::Request) == msg_result {
                if let Content::Request(request_type) = self.hub.msg().content() {
                    let error = match request_type.clone() {
                        RequestType::Register(aid) => self.register_agent(&aid),
                        RequestType::Deregister(aid) => self.deregister_agent(&aid),
                        RequestType::Suspend(aid) => self.suspend_agent(&aid),
                        RequestType::Resume(aid) => self.resume_agent(&aid),
                        RequestType::Terminate(aid) => self.terminate_agent(&aid),
                        RequestType::Search(aid) => self.search_agent(&aid),
                        _ => Err(ErrorCode::InvalidRequest),
                    };
                    self.service_req_reply_type(request_type, error);
                } else {
                    self.hub.set_msg(MessageType::NotUnderstood, Content::None);
                }
                // setting up reply
                println!("{}: REPLYING TO {}", self.hub.aid(), receiver);
                let _ = self.hub.send_to_aid(&receiver);
            }
        }
    }
    /*  pub(crate) fn modify_agent(&mut self, nickname: &str, update: Description) {
            //update agent
        }
    */

    fn service_req_reply_type(&mut self, request_type: RequestType, result: Result<(), ErrorCode>) {
        match result {
            Ok(()) => {
                let msg_content = if let RequestType::Search(aid) = request_type {
                    Content::AmsAgentDescription(AmsAgentDescription::new(aid))
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
    pub(crate) fn terminate_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        if !self.conditions.termination_condition() {
            return Err(ErrorCode::InvalidConditions(RequestType::Terminate(
                aid.clone(),
            )));
        }
        if self.search_agent(aid).is_err() {
            return Err(ErrorCode::NotFound);
        }
        let mut deck_guard = self.hub.deck_write()?;
        let state = deck_guard.agent_state(aid)?;
        if state != AgentState::Active {
            return Err(ErrorCode::InvalidStateChange(state, AgentState::Terminated));
        }
        deck_guard.modify_control_block(aid, TcbField::Quit, true)?;
        deck_guard.remove_agent(aid)
        //TBD: REMOVE HANDLE AND JOIN THREAD
    }

    pub(crate) fn suspend_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        if !self.conditions.suspension_condition() {
            return Err(ErrorCode::InvalidConditions(RequestType::Suspend(
                aid.clone(),
            )));
        }
        if self.search_agent(aid).is_err() {
            return Err(ErrorCode::NotFound);
        }
        let mut deck_guard = self.hub.deck_write()?;
        let state = deck_guard.agent_state(aid)?;
        if state != AgentState::Active {
            return Err(ErrorCode::InvalidStateChange(state, AgentState::Suspended));
        }
        deck_guard.modify_control_block(aid, TcbField::Suspend, true)
    }

    pub(crate) fn resume_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        if !self.conditions.resumption_condition() {
            return Err(ErrorCode::InvalidConditions(RequestType::Resume(
                aid.clone(),
            )));
        }
        if self.search_agent(aid).is_err() {
            return Err(ErrorCode::NotFound);
        }
        let mut deck_guard = self.hub.deck_write()?;
        let state = deck_guard.agent_state(aid)?;
        if state != AgentState::Suspended {
            return Err(ErrorCode::InvalidStateChange(state, AgentState::Suspended));
        }
        deck_guard.modify_control_block(aid, TcbField::Suspend, false)?;
        deck_guard.unpark_agent(aid)
    }

    /*  pub(crate) fn restart_agent(&mut self, nickname: &str) {
            //relaunch agent
        }
    */
}

//CAN REDUCE CODE BY CREATING: CONDITION CHECK VIA ENUM

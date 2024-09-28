use crate::{
    deck::{DeckAccess, TcbField},
    entity::{
        agent::AgentState,
        messaging::{Content, MessageType, RequestType},
        service::{Service, UserConditions},
        Description, Entity, Hub,
    },
    ErrorCode, RX,
};
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    hub: Hub,
    conditions: T,
}

impl<T: UserConditions> Entity for Ams<T> {
    fn set_aid(&mut self, aid: Description) {
        self.hub.set_aid(aid);
    }
}

impl<T: UserConditions> Service for Ams<T> {
    type Conditions = T;
    fn new(rx: RX, deck: DeckAccess, conditions: Self::Conditions) -> Self {
        let aid = Description::default();
        let hub = Hub::new(aid, rx, deck);
        Self { hub, conditions }
    }

    fn init(&mut self) {
        caravela_status!("{}: Started!", self.hub.aid())
    }

    fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        self.hub.deck()?.search_agent(aid)?;
        Ok(())
    }
    fn register_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        if !self.conditions.registration_condition() {
            return Err(ErrorCode::InvalidConditions(RequestType::Register(
                aid.clone(),
            )));
        }
        if self.search_agent(aid).is_ok() {
            return Err(ErrorCode::Duplicated);
        }
        /* NOT CURRENTLY SUPPORTED: does nothing besides checking if agent is
        already registed and only checks if the conditions allow it */
        Ok(())
    }

    fn deregister_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        if !self.conditions.deregistration_condition() {
            return Err(ErrorCode::InvalidConditions(RequestType::Deregister(
                aid.clone(),
            )));
        }
        self.hub.deck_mut()?.remove_agent(aid).and_then(|_| Ok(()))
        //TODO: FIX FLOW
    }

    fn service_function(&mut self) {
        //self.hub.set_thread();
        self.init();
        loop {
            caravela_messaging!("{}: Wating for a request...", self.hub.aid());
            let msg_result = self.hub.receive();
            //let msg_type = self.hub.receive()?;
            //let receiver = self.hub.msg().sender().clone();
            if Ok(MessageType::Request) == msg_result {
                caravela_messaging!("{}: Received Request!", self.hub.aid());
                let receiver = self.hub.msg().sender().clone();
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
                    caravela_messaging!(
                        "{}: Replying to {} from {}",
                        self.hub.aid(),
                        request_type,
                        receiver
                    );
                    self.service_req_reply_type(request_type, error);
                } else {
                    self.hub.set_msg(MessageType::NotUnderstood, Content::None);
                }
                // setting up reply
                self.hub.set_msg_receiver(receiver);
                self.hub.set_msg_sender(self.hub.aid().clone());
                let _ = self.hub.send();
                //TBD handle these possible errors;
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
        let mut deck_guard = self.hub.deck_mut()?;
        let state = deck_guard.get_agent(aid)?.control_block().agent_state();
        if state != AgentState::Active {
            return Err(ErrorCode::InvalidStateChange(state, AgentState::Terminated));
        }
        deck_guard.modify_control_block(aid, TcbField::Quit, true)?;
        deck_guard.remove_agent(aid).and_then(|_| Ok(()))
        //TBD: REMOVE HANDLE AND JOIN THREAD
        //TODO: FIX FLOW TO RELY ON TAKEDOWNS
    }

    pub(crate) fn suspend_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        if !self.conditions.suspension_condition() {
            return Err(ErrorCode::InvalidConditions(RequestType::Suspend(
                aid.clone(),
            )));
        }
        let mut deck_guard = self.hub.deck_mut()?;
        let state = deck_guard.get_agent(aid)?.control_block().agent_state();
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
        let mut deck_guard = self.hub.deck_mut()?;
        let state = deck_guard.get_agent(aid)?.control_block().agent_state();
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

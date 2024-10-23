use crate::{
    agent::AgentState,
    deck::{deck, AgentEntry},
    entity::{
        messaging::MessageType,
        service::{AmsConditions, Service},
        Description, Hub,
    },
    messaging::{ActionType, Content, Message, SyncType},
    ErrorCode, Rx,
};
use std::fmt::Debug;

#[derive(Debug)]
pub(crate) struct Ams<T: AmsConditions> {
    hap: &'static str,
    hub: Hub,
    conditions: T,
}

impl<T: AmsConditions> Service for Ams<T> {
    fn name(&self) -> String {
        format!("ams@{}", self.hap)
    }

    fn init(&mut self) {
        caravela_status!("{}: Started!", self.name())
    }

    fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        deck().read().search_agent(aid)
    }

    //fn modify_agent(&self, aid: &Description, modify: &ModifyAgent) -> Result<(), ErrorCode> {
    fn modify_agent(&self, aid: &Description, modifier: &str) -> Result<(), ErrorCode> {
        //if let ModifyAgent::State(state) = modify {
        match modifier {
            "resume" => self.resume_agent(aid),
            "suspend" => self.suspend_agent(aid),
            "terminate" => self.terminate_agent(aid),
            _ => Err(ErrorCode::InvalidContent),
        }
        //} else {
        //    Err(ErrorCode::InvalidContent)
        //}
    }

    fn register_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        /* NOT CURRENTLY SUPPORTED: does nothing besides checking if agent is
        already registed and only checks if the conditions allow it */
        deck().read().search_agent(aid)
    }

    fn deregister_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        let AgentEntry { join_handle, .. } = deck().write().remove_agent(aid)?;
        join_handle.join().map_err(|_| ErrorCode::AgentPanic)
    }

    fn service_function(&mut self) {
        self.init();
        loop {
            caravela_messaging!("{}: Wating for a request...", self.name());
            let msg_result = self.hub.receive();
            if let Ok(msg) = msg_result {
                if self.process_request(msg).is_err() {
                    //TBD handle these possible errors;
                }
            } else {
                //TBD handle these possible errors;
            }
        }
    }

    fn request_reply(
        &self,
        receiver: Description,
        message_type: MessageType,
        content: Content,
    ) -> Result<(), ErrorCode> {
        //let sender = deck().read().get_ams_address_for_hap(self.hap)?;
        let sender = deck().read().ams_aid().clone();
        caravela_messaging!(
            "{}: Replying with {} to {}",
            self.name(),
            message_type,
            receiver
        );
        let msg = Message::new(sender, receiver, message_type, content);
        self.hub.send(msg, SyncType::Blocking)
    }
}

impl<T: AmsConditions> Ams<T> {
    pub(crate) fn new(hap: &'static str, rx: Rx, conditions: T) -> Self {
        let hub = Hub::new(rx);
        Self {
            hap,
            hub,
            conditions,
        }
    }

    fn process_request(&self, msg: Message) -> Result<(), ErrorCode> {
        let receiver = msg.sender().clone();
        let content = msg.content().clone();
        if msg.message_type().clone() == MessageType::Request {
            caravela_messaging!("{}: Received Request!", self.name());
            if let Content::Action(request_type) = content.clone() {
                if self.check_conditions(&request_type) {
                    self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;
                    let req_result = self.do_request(&request_type);
                    match req_result {
                        Ok(()) => {
                            self.request_reply(receiver, MessageType::Inform, content)?;
                        }
                        Err(_) => {
                            self.request_reply(receiver, MessageType::Refuse, content)?;
                        }
                    }
                } else {
                    self.request_reply(receiver, MessageType::NotUnderstood, content)?;
                };
            };
        };
        Ok(())
    }

    fn check_conditions(&self, action: &ActionType) -> bool {
        match action {
            ActionType::Search(_) => self.conditions.search_condition(),
            ActionType::Modify(_, modifier) => {
                self.conditions.modification_condition()
                    && self.check_modification_conditions(modifier.to_lowercase().as_str())
            }
            ActionType::Register(_) => self.conditions.registration_condition(),
            ActionType::Deregister(_) => self.conditions.deregistration_condition(),
            ActionType::Other(_) => unreachable!(),
        }
    }

    //fn check_modification_conditions(&self, modify: &ModifyAgent) -> bool {
    fn check_modification_conditions(&self, modifier: &str) -> bool {
        //if let ModifyAgent::State(state) = modify {
        match modifier {
            "resume" => self.conditions.resumption_condition(),
            "suspend" => self.conditions.suspension_condition(),
            "terminate" => self.conditions.termination_condition(),
            _ => false,
        }
        //} else {
        //    false
        //}
    }

    fn do_request(&self, request: &ActionType) -> Result<(), ErrorCode> {
        match request {
            ActionType::Search(aid) => self.search_agent(&aid),
            ActionType::Modify(aid, modifier) => {
                self.modify_agent(&aid, modifier.to_string().as_str())
            }
            ActionType::Register(aid) => self.register_agent(&aid),
            ActionType::Deregister(aid) => self.deregister_agent(&aid),
            ActionType::Other(_) => Err(ErrorCode::InvalidRequest),
        }
    }

    pub(crate) fn terminate_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        deck().write().modify_agent(aid, AgentState::Terminated)?;
        self.deregister_agent(aid)
    }

    pub(crate) fn suspend_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        let deck_guard = deck().write();
        deck_guard.modify_agent(aid, AgentState::Suspended)
    }

    pub(crate) fn resume_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        let deck_guard = deck().write();
        deck_guard.modify_agent(aid, AgentState::Active)
    }

    /*  pub(crate) fn restart_agent(&mut self, nickname: &str) {
            //relaunch agent
        }
    */
}

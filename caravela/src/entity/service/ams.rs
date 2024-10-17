use crate::{
    agent::AgentState,
    deck::deck,
    entity::{
        messaging::{Content, MessageType, RequestType},
        service::{AmsConditions, Service},
        Description, Hub,
    },
    messaging::{Message, ModifyRequest, StateOps, SyncType},
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
        format!("AMS@{}", self.hap)
    }

    fn init(&mut self) {
        caravela_status!("{}: Started!", self.name())
    }

    fn search_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode> {
        let receiver = msg.sender().clone();
        let content = msg.content();
        if !self.conditions.search_condition() {
            return self.request_reply(receiver, MessageType::Refuse, content.clone());
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;

        {
            deck().read().search_agent(aid)
        }
    }
    fn register_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode> {
        let receiver = msg.sender().clone();
        let content = msg.content();
        if !self.conditions.registration_condition() {
            return self.request_reply(receiver, MessageType::Refuse, content.clone());
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;
        /* NOT CURRENTLY SUPPORTED: does nothing besides checking if agent is
        already registed and only checks if the conditions allow it */

        {
            deck().read().search_agent(aid)
        }
    }

    fn deregister_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode> {
        let receiver = msg.sender().clone();
        let content = msg.content();
        if !self.conditions.deregistration_condition() {
            return self.request_reply(receiver, MessageType::Refuse, content.clone());
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;
        {
            deck().write().remove_agent(aid).map(|_| ())
            //TBD: JOIN THREAD
            //?.join_handle
            //.join()
            //.map_err(|_| ErrorCode::AgentPanic)
        }
    }

    fn service_function(&mut self) {
        self.init();
        loop {
            caravela_messaging!("{}: Wating for a request...", self.name());
            let msg_result = self.hub.receive();
            if let Ok(msg) = msg_result {
                let receiver = msg.sender().clone();
                let content = msg.content().clone();
                let msg_result = if msg.message_type().clone() == MessageType::Request {
                    caravela_messaging!("{}: Received Request!", self.name());
                    if let Content::Request(request_type) = content.clone() {
                        let req_result = match request_type {
                            RequestType::Search(aid) => self.search_agent(&aid, msg),
                            RequestType::Modify(aid, modify) => {
                                self.modify_agent(&aid, modify, msg)
                            }
                            RequestType::Register(aid) => self.register_agent(&aid, msg),
                            RequestType::Deregister(aid) => self.deregister_agent(&aid, msg),
                        };
                        match req_result {
                            Ok(()) => {
                                self.request_reply(receiver, MessageType::Inform, content.clone())
                            }
                            Err(_) => {
                                self.request_reply(receiver, MessageType::Refuse, content.clone())
                            }
                        }
                    } else {
                        self.request_reply(receiver, MessageType::NotUnderstood, content.clone())
                    }
                } else {
                    self.request_reply(receiver, MessageType::NotUnderstood, content.clone())
                };
            } //TBD handle these possible errors;
        }
    }
    fn modify_agent(
        &self,
        aid: &Description,
        modify: ModifyRequest,
        msg: Message,
    ) -> Result<(), ErrorCode> {
        if let ModifyRequest::Ams(state) = modify {
            match state {
                StateOps::Resume => self.resume_agent(aid, msg),
                StateOps::Suspend => self.suspend_agent(aid, msg),
                StateOps::Terminate => self.terminate_agent(aid, msg),
            }
        } else {
            Err(ErrorCode::InvalidContent)
        }
    }

    fn request_reply(
        &self,
        receiver: Description,
        message_type: MessageType,
        content: Content,
    ) -> Result<(), ErrorCode> {
        let sender = deck().read().get_ams_address_for_hap(self.hap)?;
        //let sender = deck().read().ams_aid().clone();
        caravela_messaging!(
            "{}: Replying with {} to {}",
            self.name(),
            message_type,
            receiver
        );
        let msg = Message::new(sender, receiver, message_type, content);
        self.hub.send(msg, SyncType::NonBlocking)
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

    pub(crate) fn terminate_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode> {
        let receiver = msg.sender().clone();
        let content = msg.content();
        if !self.conditions.termination_condition() {
            return self.request_reply(receiver, MessageType::Refuse, content.clone());
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;
        {
            let mut deck_guard = deck().write();
            deck_guard.modify_agent(aid, AgentState::Terminated)?;
            deck_guard.remove_agent(aid).map(|_| ())
            //TBD: JOIN THREAD
        }
        //deck_guard.add_agent(aid, join_handle, priority, control_block);
        //Manage invalid transition
        //removed.join()
    }

    pub(crate) fn suspend_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode> {
        let receiver = msg.sender().clone();
        let content = msg.content();
        if !self.conditions.suspension_condition() {
            return self.request_reply(receiver, MessageType::Refuse, content.clone());
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;
        {
            let deck_guard = deck().write();
            deck_guard.modify_agent(aid, AgentState::Suspended)
        }
    }

    pub(crate) fn resume_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode> {
        let receiver = msg.sender().clone();
        let content = msg.content();
        if !self.conditions.resumption_condition() {
            return self.request_reply(receiver, MessageType::Refuse, content.clone());
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;
        {
            let deck_guard = deck().write();
            deck_guard.modify_agent(aid, AgentState::Active)
        }
    }

    /*  pub(crate) fn restart_agent(&mut self, nickname: &str) {
            //relaunch agent
        }
    */
}

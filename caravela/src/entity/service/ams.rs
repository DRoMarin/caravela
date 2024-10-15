use crate::{
    agent::AgentState,
    deck::deck,
    entity::{
        messaging::{Content, MessageType, RequestType},
        service::{AmsConditions, Service},
        Description, Hub,
    },
    messaging::{Message, SyncType},
    ErrorCode, Rx,
};
use std::fmt::Debug;

#[derive(Debug)]
pub(crate) struct Ams<T: AmsConditions> {
    hap: &'static str,
    //become Service<AMS> or Service<DF>
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
            return self.request_reply(receiver, MessageType::Refuse, content);
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;

        if { deck().read().search_agent(aid) }.is_ok() {
            self.request_reply(receiver, MessageType::Inform, content)
        } else {
            self.request_reply(receiver, MessageType::Failure, content)
        }
    }

    fn register_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode> {
        let receiver = msg.sender().clone();
        let content = msg.content();
        if !self.conditions.registration_condition() {
            return self.request_reply(receiver, MessageType::Refuse, content);
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;
        /* NOT CURRENTLY SUPPORTED: does nothing besides checking if agent is
        already registed and only checks if the conditions allow it */

        if { deck().read().search_agent(aid) }.is_ok() {
            self.request_reply(receiver, MessageType::Failure, content)
        } else {
            self.request_reply(receiver, MessageType::Inform, content)
        }
    }

    fn deregister_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode> {
        let receiver = msg.sender().clone();
        let content = msg.content();
        if !self.conditions.deregistration_condition() {
            return self.request_reply(receiver, MessageType::Refuse, content);
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;
        if {
            deck()
                .write()
                .remove_agent(aid)?
                .join_handle
                .join()
                .map_err(|_| ErrorCode::AgentPanic)
        }
        .is_ok()
        {
            self.request_reply(receiver, MessageType::Inform, content)
        } else {
            self.request_reply(receiver, MessageType::Failure, content)
        }

        //TBD: REMOVE HANDLE AND JOIN THREAD
        //TODO: FIX FLOW TO RELY ON TAKEDOWNS
    }

    fn service_function(&mut self) {
        self.init();
        loop {
            caravela_messaging!("{}: Wating for a request...", self.name());
            let msg_result = self.hub.receive();
            if let Ok(msg) = msg_result {
                let request_result = if msg.message_type() == MessageType::Request {
                    caravela_messaging!("{}: Received Request!", self.name());
                    //let receiver = self.hub.msg().sender().clone();
                    if let Content::Request(request_type) = msg.content() {
                        match request_type {
                            RequestType::Register(aid) => self.register_agent(&aid, msg),
                            RequestType::Deregister(aid) => self.deregister_agent(&aid, msg),
                            RequestType::Suspend(aid) => self.suspend_agent(&aid, msg),
                            RequestType::Resume(aid) => self.resume_agent(&aid, msg),
                            RequestType::Terminate(aid) => self.terminate_agent(&aid, msg),
                            RequestType::Search(aid) => self.search_agent(&aid, msg),
                        }
                    } else {
                        self.request_reply(
                            msg.sender().clone(),
                            MessageType::NotUnderstood,
                            msg.content(),
                        )
                    }
                } else {
                    self.request_reply(
                        msg.sender().clone(),
                        MessageType::NotUnderstood,
                        msg.content(),
                    )
                };
            } //TBD handle these possible errors;
        }
    }
    /*  pub(crate) fn modify_agent(&mut self, nickname: &str, update: Description) {
            //update agent
        }
    */
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
            return self.request_reply(receiver, MessageType::Refuse, content);
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;
        if {
            let mut deck_guard = deck().write();
            deck_guard.modify_agent(aid, AgentState::Terminated)?;
            deck_guard.remove_agent(aid)
        }
        .is_ok()
        {
            self.request_reply(receiver, MessageType::Inform, content)
        } else {
            self.request_reply(receiver, MessageType::Failure, content)
        }

        //deck_guard.add_agent(aid, join_handle, priority, control_block);
        //Manage invalid transition
        //removed.join()

        //TBD: REMOVE HANDLE AND JOIN THREAD
        //TODO: FIX FLOW TO RELY ON TAKEDOWNS
    }

    pub(crate) fn suspend_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode> {
        let receiver = msg.sender().clone();
        let content = msg.content();
        if !self.conditions.suspension_condition() {
            return self.request_reply(receiver, MessageType::Refuse, content);
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;
        if {
            let deck_guard = deck().write();
            deck_guard.modify_agent(aid, AgentState::Suspended)
        }
        .is_ok()
        {
            self.request_reply(receiver, MessageType::Inform, content)
        } else {
            self.request_reply(receiver, MessageType::Failure, content)
        }
    }

    pub(crate) fn resume_agent(&self, aid: &Description, msg: Message) -> Result<(), ErrorCode> {
        let receiver = msg.sender().clone();
        let content = msg.content();
        if !self.conditions.resumption_condition() {
            return self.request_reply(receiver, MessageType::Refuse, content);
        }
        self.request_reply(receiver.clone(), MessageType::Agree, content.clone())?;
        if {
            let deck_guard = deck().write();
            deck_guard.modify_agent(aid, AgentState::Active)
        }
        .is_ok()
        {
            self.request_reply(receiver, MessageType::Inform, content)
        } else {
            self.request_reply(receiver, MessageType::Failure, content)
        }
    }

    /*  pub(crate) fn restart_agent(&mut self, nickname: &str) {
            //relaunch agent
        }
    */
}

//CAN REDUCE CODE BY CREATING: CONDITION CHECK VIA ENUM

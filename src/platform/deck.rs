use std::{
    collections::HashMap,
    sync::{
        atomic::Ordering,
        mpsc::{SendError, TrySendError},
        Arc,
    },
    thread::JoinHandle,
};

use crate::platform::{
    agent::ControlBlock,
    entity::messaging::{Content, Message, MessageType},
    entity::Description,
    AgentState, ErrorCode, MAX_SUBSCRIBERS,
};

pub(crate) type Directory = HashMap<String, Description>; //can be expanded into different dir types for agents, AMS or DF if present
pub(crate) type ControlBlockDirectory = HashMap<String, Arc<ControlBlock>>;
pub(crate) type HandleDirectory = HashMap<String, JoinHandle<()>>;
//pub(crate) type StateDirectory = HashMap<String, AgentState>;
//pub(crate) type AddressDirectory = HashMap<String, Sender<Message>>;

pub enum SyncType {
    Blocking,
    NonBlocking,
}
enum SendResult {
    Blocking(Result<(), SendError<Message>>),
    NonBlocking(Result<(), TrySendError<Message>>),
}

pub(crate) enum TcbField {
    Suspend,
    Quit,
}

pub struct Deck {
    pub(crate) handle_directory: HandleDirectory, //accessed only by AMS
    pub(crate) white_pages_directory: Directory,  //accessed only by AMS
    pub(crate) control_block_directory: ControlBlockDirectory, //accessed only by AMS
                                                  //pub(crate) state_directory: StateDirectory,   //modified only by Agents, accessed by All
}

impl Deck {
    pub(crate) fn new() -> Self {
        let handle_directory: HandleDirectory = HandleDirectory::with_capacity(MAX_SUBSCRIBERS);
        let control_block_directory: ControlBlockDirectory =
            ControlBlockDirectory::with_capacity(MAX_SUBSCRIBERS);
        let white_pages_directory: Directory = Directory::with_capacity(MAX_SUBSCRIBERS);
        //let state_directory: StateDirectory = StateDirectory::with_capacity(MAX_SUBSCRIBERS);
        Self {
            handle_directory,
            control_block_directory,
            white_pages_directory,
            //state_directory,
        }
    }
    pub(crate) fn add_new_agent(
        &mut self,
        description: Description,
        handle: JoinHandle<()>,
        cb: Arc<ControlBlock>,
    ) {
        self.handle_directory.insert(description.get_name(), handle);
        self.control_block_directory
            .insert(description.get_name(), cb);
        self.white_pages_directory
            .insert(description.get_name(), description);
    }
    pub(crate) fn search_agent(&self, nickname: &str) -> ErrorCode {
        if self.white_pages_directory.contains_key(nickname) {
            ErrorCode::Found
        } else {
            ErrorCode::NotFound
        }
    }
    pub(crate) fn insert_agent(&mut self, nickname: &str, description: Description) -> ErrorCode {
        if self.white_pages_directory.len().eq(&MAX_SUBSCRIBERS) {
            ErrorCode::ListFull
        } else {
            self.white_pages_directory
                .insert(nickname.to_string(), description);
            ErrorCode::NoError
        }
    }
    //pub(crate) fn modify_agent(&self) -> ErrorCode {}
    pub(crate) fn remove_agent(&mut self, nickname: &str) -> ErrorCode {
        if self.white_pages_directory.remove(nickname).is_none() {
            return ErrorCode::Invalid;
        }

        if self.control_block_directory.remove(nickname).is_none() {
            return ErrorCode::Invalid;
        }
        ErrorCode::NoError
    }
    pub(crate) fn get_agent(&self, nickname: &str) -> Description {
        self.white_pages_directory.get(nickname).unwrap().clone()
    }
    pub(crate) fn unpark_agent(&mut self, nickname: &str) {
        self.handle_directory
            .entry(nickname.to_string())
            .and_modify(|handle| handle.thread().unpark());
    }
    pub(crate) fn modify_control_block(&mut self, nickname: &str, field: TcbField, val: bool) {
        match field {
            TcbField::Suspend => self
                .control_block_directory
                .entry(nickname.to_string())
                .and_modify(|x| x.suspend.store(val, Ordering::Relaxed)),

            TcbField::Quit => self
                .control_block_directory
                .entry(nickname.to_string())
                .and_modify(|x| x.quit.store(val, Ordering::Relaxed)),
        };
    }
    pub(crate) fn get_agent_state(&self, nickname: &str) -> AgentState {
        let block = self.control_block_directory.get(nickname).unwrap();
        if block.suspend.load(Ordering::Relaxed) {
            AgentState::Suspended
        } else if block.wait.load(Ordering::Relaxed) {
            AgentState::Waiting
        } else if block.active.load(Ordering::Relaxed) {
            AgentState::Active
        } else {
            AgentState::Initiated
        }
    }
    pub(crate) fn send(
        &self,
        sender_aid: Description,
        receiver: &str,
        msg: Message,
        sync: SyncType,
    ) -> ErrorCode {
        let receiver_aid = match self.white_pages_directory.get(receiver) {
            Some(x) => x.clone(),
            None => return ErrorCode::NotRegistered,
        };
        self.send_to_aid(sender_aid, receiver_aid, msg, sync)
    }
    pub(crate) fn send_to_aid(
        &self,
        sender_aid: Description,
        receiver_aid: Description,
        msg: Message,
        sync: SyncType,
    ) -> ErrorCode {
        //check memberships and roles
        let address = receiver_aid.get_address();
        let result = match sync {
            SyncType::Blocking => SendResult::Blocking(address.send(msg)),
            SyncType::NonBlocking => SendResult::NonBlocking(address.try_send(msg)),
        };
        let error_code = match result {
            SendResult::Blocking(Ok(_)) => ErrorCode::NoError,
            SendResult::NonBlocking(Ok(_)) => ErrorCode::NoError,
            SendResult::Blocking(Err(SendError(_))) => ErrorCode::Invalid,
            SendResult::NonBlocking(Err(error)) => match error {
                TrySendError::Disconnected(_) => ErrorCode::Invalid, //LIST MAY BE OUTDATED
                TrySendError::Full(_) => ErrorCode::Timeout,
            },
        };
        error_code
    }
    /*fn prepare_message(
        sender_aid: Description,
        receiver_aid: Description,
        msg_type: MessageType,
        content: Content,
    ) -> Message {
        let mut msg = Message::new();
        msg.set_sender(sender_aid);
        msg.set_receiver(receiver_aid);
        msg.set_type(msg_type);
        msg.set_content(content);
        msg
    }*/
    /* add service request protocols */
}

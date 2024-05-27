use crate::{
    entity::{agent::AgentState, messaging::Message, Description},
    platform::environment::{agent_control_block, agent_thread_handle},
    ErrorCode, MAX_SUBSCRIBERS,
};
use std::{
    collections::HashSet,
    sync::{
        atomic::Ordering,
        mpsc::{SendError, TrySendError},
    },
};

pub(crate) type AidDirectory = HashSet<Description>;

#[derive(Debug)]
pub enum SyncType {
    Blocking,
    #[allow(dead_code)]
    NonBlocking, //USE?
}

#[derive(Debug)]
enum SendResult {
    Blocking(Result<(), SendError<Message>>),
    NonBlocking(Result<(), TrySendError<Message>>),
}

#[derive(Debug)]
pub(crate) enum TcbField {
    Suspend,
    Quit,
}

#[derive(Debug)]
pub struct Deck {
    pub(crate) wp_directory: AidDirectory,
}

impl Deck {
    pub(crate) fn new() -> Self {
        let wp_directory: AidDirectory = AidDirectory::with_capacity(MAX_SUBSCRIBERS);
        Self { wp_directory }
    }

    pub(crate) fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        if self.wp_directory.contains(aid) {
            Ok(())
        } else {
            Err(ErrorCode::NotFound)
        }
    }

    pub(crate) fn insert_agent(&mut self, aid: Description) -> Result<(), ErrorCode> {
        if let Err(ErrorCode::NotFound) = self.search_agent(&aid) {
            if self.wp_directory.len().eq(&MAX_SUBSCRIBERS) {
                Err(ErrorCode::ListFull)
            } else {
                self.wp_directory.insert(aid);
                Ok(())
            }
        } else {
            Err(ErrorCode::Duplicated)
        }
    }

    //pub(crate) fn modify_agent(&self) -> ErrorCode {}

    pub(crate) fn remove_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        self.search_agent(aid)?;
        self.wp_directory.remove(aid);
        Ok(())
    }

    pub(crate) fn unpark_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        self.search_agent(aid)?;
        let entry = agent_thread_handle(aid)?;
        entry.unpark();
        Ok(())
    }

    pub(crate) fn modify_control_block(
        &mut self,
        aid: &Description,
        field: TcbField,
        val: bool,
    ) -> Result<(), ErrorCode> {
        self.search_agent(aid)?;
        let control_block = agent_control_block(aid)?;
        match field {
            TcbField::Suspend => control_block.suspend.store(val, Ordering::Relaxed),

            TcbField::Quit => control_block.quit.store(val, Ordering::Relaxed),
        };
        Ok(())
    }

    pub(crate) fn agent_state(&self, aid: &Description) -> Result<AgentState, ErrorCode> {
        self.search_agent(aid)?;
        let block = agent_control_block(aid)?;
        //let block = entry.control_block;
        let state = if block.suspend.load(Ordering::Relaxed) {
            AgentState::Suspended
        } else if block.wait.load(Ordering::Relaxed) {
            AgentState::Waiting
        } else if block.active.load(Ordering::Relaxed) {
            AgentState::Active
        } else {
            AgentState::Initiated
        };
        Ok(state)
    }

    pub(crate) fn send_msg(&self, msg: Message, sync: SyncType) -> Result<(), ErrorCode> {
        //check memberships and roles
        let receiver_aid = msg.receiver();
        if receiver_aid.nickname() != "AMS" {
            self.search_agent(receiver_aid)?;
        }
        let address = receiver_aid.address()?;
        let result = match sync {
            SyncType::Blocking => SendResult::Blocking(address.send(msg)),
            SyncType::NonBlocking => SendResult::NonBlocking(address.try_send(msg)),
        };
        match result {
            SendResult::Blocking(Ok(_)) => Ok(()),
            SendResult::NonBlocking(Ok(_)) => Ok(()),
            SendResult::Blocking(Err(SendError(_))) => Err(ErrorCode::Disconnected),
            SendResult::NonBlocking(Err(error)) => match error {
                TrySendError::Disconnected(_) => Err(ErrorCode::Disconnected), //LIST MAY BE OUTDATED
                TrySendError::Full(_) => Err(ErrorCode::ChannelFull),
            },
        }
        //FIX
    }
    /* add service request protocols */
}

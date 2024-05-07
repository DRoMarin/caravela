use crate::{
    entity::{
        agent::{AgentState, ControlBlock},
        messaging::Message,
        Description,
    },
    platform::env::{platform_env, DirectoryEntry},
    ErrorCode, MAX_SUBSCRIBERS,
};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::Ordering,
        mpsc::{SendError, SyncSender, TrySendError},
        Arc,
    },
    thread::JoinHandle,
};

//pub(crate) type AddressDirectory = HashMap<String, SyncSender<Message>>;
//pub(crate) type HandleDirectory = HashMap<String, JoinHandle<()>>;
//pub(crate) type ControlBlockDirectory = HashMap<String, Arc<ControlBlock>>;

//can be expanded into different dir types for agents, AMS or DF if present
pub(crate) type AidDirectory = HashSet<Description>;
//TBD fuse all these dirs together
//pub(crate) type AgentDirectory = HashMap<String, Description>;
//pub(crate) type AgentDirectory = HashSet<Description>;
pub(crate) type CtrlBlkDirectory = HashMap<Description, Arc<ControlBlock>>;
pub(crate) type HandleDirectory = HashMap<Description, JoinHandle<()>>;
pub(crate) type AddressDirectory = HashMap<Description, SyncSender<Message>>;

/*pub(crate) struct DirectoryEntry {
    join_handle: JoinHandle<()>,
    control_block: Arc<ControlBlock>,
    tx_address: SyncSender<Message>,
}*/
//pub(crate) type PlatformDirectory = HashMap<Description, DirectoryEntry>;
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
    //pub(crate) name_directory: NameDirectory,
    //pub(crate) handle_directory: HandleDirectory,
    //pub(crate) address_directory: AddressDirectory,
    //pub(crate) ctrl_blk_directory: CtrlBlkDirectory,
    pub(crate) wp_directory: AidDirectory,
    //pub(crate) platform_directory: PlatformDirectory,
    //pub(crate) platform_directory: &'static OnceLock<RwLock<PlatformDirectory>>,
}

impl Deck {
    pub(crate) fn new() -> Self {
        //       let name_directory: NameDirectory = NameDirectory::with_capacity(MAX_SUBSCRIBERS);
        //       let handle_directory: HandleDirectory = HandleDirectory::with_capacity(MAX_SUBSCRIBERS);
        //       let address_directory: AddressDirectory = AddressDirectory::with_capacity(MAX_SUBSCRIBERS);
        //       let ctrl_blk_directory: CtrlBlkDirectory = CtrlBlkDirectory::with_capacity(MAX_SUBSCRIBERS);
        //       let white_pages_directory: AidDirectory = AidDirectory::with_capacity(MAX_SUBSCRIBERS);

        //Could be replace by Platform:ENV
        //let platform_directory = &PLATFORM_ENV;
        //let platform_directory: PlatformDirectory =
        //    PlatformDirectory::with_capacity(MAX_SUBSCRIBERS);
        let wp_directory: AidDirectory = AidDirectory::with_capacity(MAX_SUBSCRIBERS);
        Self {
            //name_directory,
            //handle_directory,
            //address_directory,
            //ctrl_blk_directory,
            //white_pages_directory,
            wp_directory,
            //platform_directory,
        }
    }
    /*
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
    */
    pub(crate) fn platform_env_entry_mut_ref(
        &self,
        aid: &Description,
    ) -> Result<&mut DirectoryEntry, ErrorCode> {
        //if self.white_pages_directory.contains(aid) {
        if let Ok(mut lock) = platform_env().write() {
            if let Some(entry) = lock.get_mut(aid) {
                Ok(entry)
            } else {
                Err(ErrorCode::NotFound)
            }
        } else {
            /* for key in self.white_pages_directory.keys() {
                println!("{}", key)
            }*/
            Err(ErrorCode::PoisonedLock)
        }
    }

    pub(crate) fn search_agent(&self, aid: &Description) -> Result<(), ErrorCode> {
        if self.wp_directory.contains(aid) {
            Ok(())
        } else {
            Err(ErrorCode::NotFound)
        }
    }
    pub(crate) fn insert_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        if let Err(ErrorCode::NotFound) = self.search_agent(aid) {
            if self.wp_directory.len().eq(&MAX_SUBSCRIBERS) {
                Err(ErrorCode::ListFull)
            } else {
                self.wp_directory.insert(aid.to_owned());
                Ok(())
            }
        } else {
            Err(ErrorCode::Duplicated)
        }
    }

    //pub(crate) fn modify_agent(&self) -> ErrorCode {}

    pub(crate) fn remove_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        todo!("FILL THE REMOVE AGENT FN IN DECK")
    }

    /*pub(crate) fn agent_aid(&self, name: &str) -> Option<Description> {
        self.white_pages_directory.get(name).cloned()
    }*/

    pub(crate) fn unpark_agent(&mut self, aid: &Description) -> Result<(), ErrorCode> {
        //let aid = self.search_agent(handle)?;
        self.search_agent(aid)?;
        let entry = self.platform_env_entry_mut_ref(aid)?;
        entry.join_handle.thread().unpark();
        Ok(())

        /*self.handle_directory
        .entry(aid.to_owned())
        .and_modify(|handle| handle.thread().unpark());*/
    }

    pub(crate) fn modify_control_block(
        &mut self,
        aid: &Description,
        field: TcbField,
        val: bool,
    ) -> Result<(), ErrorCode> {
        self.search_agent(aid)?;
        let entry = self.platform_env_entry_mut_ref(aid)?;
        match field {
            TcbField::Suspend => entry.control_block.suspend.store(val, Ordering::Relaxed),

            TcbField::Quit => entry.control_block.quit.store(val, Ordering::Relaxed),
            /*TcbField::Suspend => self
                .ctrl_blk_directory
                .entry(aid.to_owned())
                .and_modify(|x| x.suspend.store(val, Ordering::Relaxed)),

            TcbField::Quit => self
                .ctrl_blk_directory
                .entry(aid.to_owned())
                .and_modify(|x| x.quit.store(val, Ordering::Relaxed)),*/
        };
        Ok(())
    }

    pub(crate) fn agent_state(&self, aid: &Description) -> Result<AgentState, ErrorCode> {
        self.search_agent(aid)?;
        let entry = self.platform_env_entry_mut_ref(aid)?;
        let block = entry.control_block;
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
        //dbg!(msg.clone());
        //dbg!(&self.address_directory);
        let receiver_aid = msg.receiver();
        self.search_agent(receiver_aid)?;
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

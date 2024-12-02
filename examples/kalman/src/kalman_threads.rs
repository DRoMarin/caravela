use std::{
    collections::HashMap,
    error::Error,
    sync::{
        mpsc::{self, SyncSender},
        Arc, RwLock,
    },
    thread,
};
use thread_priority::*;

use crate::{Q, R0, R1, R2};

type SenderList = Arc<RwLock<HashMap<&'static str, SyncSender<String>>>>;

mod correct;
mod log;
mod predict;
mod sense;

pub fn pure_main(
    in_filepath: &'static str,
    out_filepath: &'static str,
) -> Result<(), Box<dyn Error>> {
    let (sense_tx, sense_rx) = mpsc::sync_channel::<String>(1);
    let (predict_tx, predict_rx) = mpsc::sync_channel::<String>(1);
    let (correct_tx, correct_rx) = mpsc::sync_channel::<String>(1);
    let (logging_tx, logging_rx) = mpsc::sync_channel::<String>(1);

    let mut senders = HashMap::new();

    senders.insert("sense", sense_tx);
    senders.insert("predict", predict_tx);
    senders.insert("correct", correct_tx);
    senders.insert("logging", logging_tx);

    let lock_senders = Arc::new(RwLock::from(senders));

    let sense_priority = ThreadPriority::Crossplatform(3.try_into()?);
    let predict_priority = ThreadPriority::Crossplatform(2.try_into()?);
    let correct_priority = ThreadPriority::Crossplatform(2.try_into()?);
    let logging_priority = ThreadPriority::Crossplatform(1.try_into()?);

    let sense_senders = lock_senders.clone();
    let predict_senders = lock_senders.clone();
    let correct_senders = lock_senders.clone();
    let logging_senders = lock_senders.clone();

    let _sense_thread = thread::Builder::new().spawn_with_priority(
        sense_priority,
        move |_| -> Result<(), Box<dyn Error + Send>> {
            let _ = sense::sensor(in_filepath, sense_rx, sense_senders);
            Ok(())
        },
    )?;

    let _predict_thread = thread::Builder::new().spawn_with_priority(
        predict_priority,
        move |_| -> Result<(), Box<dyn Error + Send>> {
            let _ = predict::predict(Q, predict_rx, predict_senders);
            Ok(())
        },
    )?;

    let _correct_thread = thread::Builder::new().spawn_with_priority(
        correct_priority,
        move |_| -> Result<(), Box<dyn Error + Send>> {
            let _ = correct::correct(R0, R1, R2, correct_rx, correct_senders);
            Ok(())
        },
    )?;

    let _logging_thread = thread::Builder::new().spawn_with_priority(
        logging_priority,
        move |_| -> Result<(), Box<dyn Error + Send>> {
            let _ = log::logging(out_filepath, logging_rx, logging_senders);
            Ok(())
        },
    )?;
    thread::park();
    Ok(())
}

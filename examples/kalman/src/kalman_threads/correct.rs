use std::sync::mpsc::{Receiver, SyncSender};

use crate::kalman;

fn main_loop(
    rx: &Receiver<String>,
    sense: &SyncSender<String>,
    logging: &SyncSender<String>,
    r: &kalman::MatrixType,
) -> Result<(), String> {
    // get predicted state
    let state_covar_msg = rx.recv().map_err(|e| e.to_string())?;
    let state_covar = serde_json::from_str(state_covar_msg.as_str()).map_err(|e| e.to_string())?;

    // request measurements
    sense
        .send("accelerometer".to_string())
        .map_err(|e| e.to_string())?;

    // get measurements
    let z_msg = rx.recv().map_err(|e| e.to_string())?;
    let z = serde_json::from_str(z_msg.as_str()).map_err(|e| e.to_string())?;

    // correct
    let correction = kalman::correct(&state_covar, &z, &r).map_err(|e| e.to_string())?;

    // get string
    let state_covar_string = correction.to_json().map_err(|e| e.to_string())?;

    // send state
    logging.send(state_covar_string).map_err(|e| e.to_string())
}
pub fn correct(
    r0: kalman::DataType,
    r1: kalman::DataType,
    r2: kalman::DataType,
    rx: Receiver<String>,
    senders: super::SenderList,
) -> Result<(), String> {
    println!("START CORRECTION THREAD");

    let (sense, logging) = {
        let lock = senders.read().map_err(|e| e.to_string())?;
        (
            lock.get("sense").unwrap().clone(),
            lock.get("logging").unwrap().clone(),
        )
    };
    let r = kalman::MatrixType::from_diagonal(&kalman::VectorType::new(r0, r1, r2));
    loop {
        let res = main_loop(&rx, &sense, &logging, &r);
        if let Err(e) = res {
            println!("CORRECT: {}", e.as_str());
        }
    }
}

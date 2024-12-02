use std::sync::mpsc::{Receiver, SyncSender};

use crate::kalman;

fn main_loop(
    rx: &Receiver<String>,
    sense: &SyncSender<String>,
    correct: &SyncSender<String>,
    q: &kalman::MatrixType,
) -> Result<(), String> {
    // get predicted state
    let state_covar_msg = rx.recv().map_err(|e| e.to_string())?;
    let state_covar = serde_json::from_str(state_covar_msg.as_str()).map_err(|e| e.to_string())?;

    // request measurements
    sense
        .send("gyroscope".to_string())
        .map_err(|e| e.to_string())?;

    // get measurements
    let u_msg = rx.recv().map_err(|e| e.to_string())?;
    let u: kalman::VectorType = serde_json::from_str(u_msg.as_str()).map_err(|e| e.to_string())?;

    // correct
    let prediction = kalman::predict(&state_covar, kalman::DT, &u, &q);

    // get string
    let state_covar_string = prediction.to_json()?;

    // send state
    correct.send(state_covar_string).map_err(|e| e.to_string())
}

pub fn predict(
    q: kalman::DataType,
    rx: Receiver<String>,
    senders: super::SenderList,
) -> Result<(), String> {
    println!("START PREDICTION THREAD");
    let (sense, correct) = {
        let lock = senders.read().map_err(|e| e.to_string())?;
        (
            lock.get("sense").unwrap().clone(),
            lock.get("correct").unwrap().clone(),
        )
    };
    let q_mat = kalman::MatrixType::from_diagonal_element(q);
    loop {
        let res = main_loop(&rx, &sense, &correct, &q_mat);
        if let Err(e) = res {
            println!("PREDICT: {}", e.as_str());
        }
    }
}

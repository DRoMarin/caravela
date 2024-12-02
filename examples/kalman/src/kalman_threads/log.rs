use std::sync::mpsc::{Receiver, SyncSender};

use crate::kalman;

struct Logger {
    state_covar: kalman::StateCovariance,
    writer: csv::Writer<std::fs::File>,
}

impl Logger {
    pub fn new(filepath: &'static str) -> Result<Self, String> {
        let file = std::fs::File::create(filepath).map_err(|e| e.to_string())?;
        //let file = std::fs::OpenOptions::new().read(true).write(false).open(filepath).map_err(|e| e.to_string())?;
        let writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);
        let state_covar = kalman::StateCovariance::new();
        Ok(Self {
            state_covar,
            writer,
        })
    }
    fn set_header(&mut self) -> Result<(), String> {
        self.writer
            .write_record(["roll", "pitch", "yaw"])
            .map_err(|e| e.to_string())
    }
    fn write_state(&mut self) -> Result<(), String> {
        self.writer
            .serialize(self.state_covar.x())
            .map_err(|e| e.to_string())?;
        self.writer.flush().map_err(|e| e.to_string())
    }
    fn update_state_covar(&mut self, new: kalman::StateCovariance) {
        self.state_covar = new;
    }
}

fn main_loop(
    rx: &Receiver<String>,
    predict: &SyncSender<String>,
    logger: &mut Logger,
) -> Result<(), String> {
    //previous values
    let previous = &logger.state_covar;
    let previous_string = previous.to_json()?;

    // send state covariance
    //println!("sending to predict from logging");
    predict.send(previous_string).map_err(|e| e.to_string())?;

    // wait for estimated state covariance
    let new = rx.recv().map_err(|e| e.to_string())?;
    let new_state_covar: kalman::StateCovariance =
        serde_json::from_str(new.as_str()).map_err(|e| e.to_string())?;

    // serialize and save state
    //println!("writing!");
    logger.update_state_covar(new_state_covar);
    logger.write_state()
}

pub fn logging(
    filepath: &'static str,
    rx: Receiver<String>,
    senders: super::SenderList,
) -> Result<(), String> {
    let predict = {
        let lock = senders.read().map_err(|e| e.to_string())?;
        lock.get("predict").unwrap().clone()
    };

    let res = Logger::new(filepath);
    let mut logger = match res {
        Ok(x) => x,
        Err(e) => {
            panic!("{:?}", e)
        } //println!("{}", res.to_string());
          //panic!("aaaaaaaaaaaaa");
    };
    logger.set_header()?;

    println!("START LOGGING THREAD");

    loop {
        let res = main_loop(&rx, &predict, &mut logger);
        if let Err(e) = res {
            println!("LOG: {}", e.as_str());
        }
    }
}

use std::sync::mpsc::{Receiver, SyncSender};

use crate::imu_data;

struct Sensor {
    reader: csv::Reader<std::fs::File>,
    data: imu_data::ImuData,
    update: bool,
}
impl Sensor {
    pub fn new(filepath: &'static str) -> Result<Self, String> {
        let file = std::fs::File::open(filepath).map_err(|e| e.to_string())?;
        let reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);
        let data = imu_data::ImuData::default();
        Ok(Self {
            reader,
            data,
            update: true,
        })
    }
    fn update_data(&mut self) -> Result<(), String> {
        let imu_data =
            imu_data::read_imu_from_reader(&mut self.reader).map_err(|e| e.to_string())?;
        if imu_data.timestamp().fract() == 0.0 {
            println!("{:?}", imu_data);
        }
        self.data = imu_data;
        self.update = false;
        Ok(())
    }
    fn serialize_gyro(&self) -> Result<String, String> {
        serde_json::to_string(self.data.gyro()).map_err(|e| e.to_string())
    }

    fn serialize_accel(&self) -> Result<String, String> {
        serde_json::to_string(self.data.accel()).map_err(|e| e.to_string())
    }
}

fn main_loop(
    sensor: &mut Sensor,
    rx: &Receiver<String>,
    correct: &SyncSender<String>,
    predict: &SyncSender<String>,
) -> Result<(), String> {
    if sensor.update {
        sensor.update_data()?;
    }
    let msg = rx.recv().map_err(|e| e.to_string())?;

    // format data
    match msg.as_str() {
        "accelerometer" => {
            let content = sensor.serialize_accel()?;
            sensor.update = true;
            correct.send(content).map_err(|e| e.to_string())
        }
        "gyroscope" => {
            let content = sensor.serialize_gyro()?;
            predict.send(content).map_err(|e| e.to_string())
        }
        _ => Ok(()),
    }
}

pub fn sensor(
    filepath: &'static str,
    rx: Receiver<String>,
    senders: super::SenderList,
) -> Result<(), String> {
    let lock = senders.read().map_err(|e| e.to_string())?;
    let correct = lock.get("correct").ok_or("not found")?;
    let predict = lock.get("predict").ok_or("not found")?;

    let res = Sensor::new(filepath);
    let mut sensor = match res {
        Ok(x) => x,
        Err(e) => {
            panic!("{}", e)
        } 
    };

    println!("START SENSING THREAD");
    loop {
        let res = main_loop(&mut sensor, &rx, &correct, &predict);
        if let Err(e) = res {
            println!("SENSOR: {}", e.as_str());
        }
        if sensor.reader.is_done() {
            break;
        }
    }
    Ok(())
}

use csv::Reader;
use nalgebra::Vector3;
use std::{fmt::Display, fs::File, num::ParseFloatError};

use crate::kalman::{DataType, VectorType};
#[allow(unused)]
#[derive(Debug, Default)]
pub struct ImuData(DataType, VectorType, VectorType, VectorType);

impl ImuData {
    pub fn timestamp(&self) -> &DataType {
        &self.0
    }
    pub fn accel(&self) -> &VectorType {
        &self.1
    }
    pub fn gyro(&self) -> &VectorType {
        &self.2
    }
    #[allow(unused)]
    pub fn mag(&self) -> &VectorType {
        &self.3
    }
}

#[derive(Debug)]
pub enum ImuDataError {
    Empty,
    ParseError,
}

impl From<ParseFloatError> for ImuDataError {
    fn from(_value: ParseFloatError) -> Self {
        ImuDataError::ParseError
    }
}

impl Display for ImuDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImuDataError::Empty => write!(f, "no more samples left"),
            ImuDataError::ParseError => write!(f, "could not parse sample"),
        }
    }
}

pub fn read_imu_from_reader(reader: &mut Reader<File>) -> Result<ImuData, ImuDataError> {
    let mut record = csv::StringRecord::new();
    if let Ok(true) = reader.read_record(&mut record) {
        let timestamp: DataType = record[0].parse()?;
        let accel_x: DataType = record[1].parse()?;
        let accel_y: DataType = record[2].parse()?;
        let accel_z: DataType = record[3].parse()?;
        let gyro_x: DataType = record[4].parse()?;
        let gyro_y: DataType = record[5].parse()?;
        let gyro_z: DataType = record[6].parse()?;
        //let mag_x: f64 = record[7].parse()?;
        //let mag_y: f64 = record[8].parse()?;
        //let mag_z: f64 = record[9].parse()?;
        //
        let accel = Vector3::new(accel_x, accel_y, accel_z);
        let gyro = Vector3::new(gyro_x, gyro_y, gyro_z);
        //let mag = Vector3::new(mag_x, mag_y, mag_z);
        let mag = Vector3::zeros();
        Ok(ImuData(timestamp, accel, gyro, mag))
    } else {
        Err(ImuDataError::Empty)
    }
}

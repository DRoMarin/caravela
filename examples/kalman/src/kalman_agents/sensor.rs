use crate::imu_data;
use caravela::{
    agent::{Agent, AgentBuildParam},
    behavior::Behavior,
    make_agent_with_param,
    messaging::*,
    ErrorCode,
};

#[derive(Debug)]
pub struct SensorParams {
    reader: csv::Reader<std::fs::File>,
    data: imu_data::ImuData,
    update: bool,
}

make_agent_with_param!(pub Sensor, SensorParams);

impl SensorParams {
    pub fn new(filepath: &'static str) -> Result<Self, ErrorCode> {
        let file = std::fs::File::open(filepath).map_err(|e| ErrorCode::Other(e.to_string()))?;
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
    fn get_data(&mut self) -> Result<(), ErrorCode> {
        let imu_data = imu_data::read_imu_from_reader(&mut self.reader)
            .map_err(|e| ErrorCode::Other(e.to_string()))?;
        if imu_data.timestamp().fract() == 0.0 {
            println!("{:?}", imu_data);
        }
        self.data = imu_data;
        self.update = false;
        Ok(())
    }
    fn serialize_gyro(&self) -> Result<String, ErrorCode> {
        serde_json::to_string(self.data.gyro()).map_err(|e| ErrorCode::Other(e.to_string()))
    }

    fn serialize_accel(&self) -> Result<String, ErrorCode> {
        serde_json::to_string(self.data.accel()).map_err(|e| ErrorCode::Other(e.to_string()))
    }
}

impl Behavior for Sensor {
    fn setup(&mut self) -> Result<(), ErrorCode> {
        println!("START SENSOR AGENT");
        Ok(())
    }
    fn action(&mut self) -> Result<(), ErrorCode> {
        let param = &mut self.param;
        if param.update {
            param.get_data()?;
        }

        let msg = self.agent.receive()?;
        msg.message_type().is_message_type(&MessageType::Request)?;

        // format data
        let content_string = match msg.content().to_string().as_str() {
            "Send Accelerometer" => {
                param.update = true;
                param.serialize_accel()?
            }
            "Send Gyroscope" => param.serialize_gyro()?,
            _ => "".to_string(),
        };

        self.agent.send_to_aid(
            msg.sender().clone(),
            MessageType::Inform,
            Content::Expression(content_string),
        )?;

        Ok(())
    }
    fn done(&mut self) -> bool {
        self.param.reader.is_done()
    }
}

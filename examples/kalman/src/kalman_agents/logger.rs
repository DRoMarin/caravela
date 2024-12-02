use crate::kalman;
use caravela::{
    agent::{Agent, AgentBuildParam},
    behavior::Behavior,
    make_agent_with_param,
    messaging::*,
    ErrorCode,
};

#[derive(Debug)]
pub struct LoggerParams {
    state_covar: kalman::StateCovariance,
    writer: csv::Writer<std::fs::File>,
}

impl LoggerParams {
    pub fn new(filepath: &'static str) -> Result<Self, ErrorCode> {
        let file = std::fs::File::create(filepath).map_err(|e| ErrorCode::Other(e.to_string()))?;
        let writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);
        let state_covar = kalman::StateCovariance::new();
        Ok(Self {
            writer,
            state_covar,
        })
    }
    fn set_header(&mut self) -> Result<(), ErrorCode> {
        self.writer
            .write_record(["roll", "pitch", "yaw"])
            .map_err(|e| ErrorCode::Other(e.to_string()))
    }
    fn write_state(&mut self) -> Result<(), ErrorCode> {
        self.writer
            .serialize(self.state_covar.x())
            .map_err(|e| ErrorCode::Other(e.to_string()))?;
        self.writer
            .flush()
            .map_err(|e| ErrorCode::Other(e.to_string()))
    }
    fn update_state_covar(&mut self, new: kalman::StateCovariance) {
        self.state_covar = new;
    }
}

make_agent_with_param!(pub Logger, LoggerParams);

impl Behavior for Logger {
    fn setup(&mut self) -> Result<(), ErrorCode> {
        println!("START LOGGER AGENT");
        self.param.set_header()?;
        self.agent.add_contact("AgentPredictor")
    }
    fn action(&mut self) -> Result<(), ErrorCode> {
        let param = &mut self.param;

        // previous values
        let previous = &param.state_covar;
        let previous_string = previous.to_json().map_err(ErrorCode::Other)?;
        let previous_content = Content::Expression(previous_string);

        //self.agent.wait(1);
        // send state covariance
        self.agent
            .send_to("AgentPredictor", MessageType::Inform, previous_content)?;

        // wait for estimated state covariance
        let new = self.agent.receive()?;
        new.message_type().is_message_type(&MessageType::Inform)?;
        let new_string = new.content().to_string();
        let new_state_covar: kalman::StateCovariance = serde_json::from_str(new_string.as_str())
            .or(Err(ErrorCode::InvalidContent(new_string)))?;

        // serialize and save state
        param.update_state_covar(new_state_covar);
        param.write_state()
    }
}

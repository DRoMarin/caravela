use crate::kalman;
use caravela::{
    agent::{Agent, AgentBuildParam},
    behavior::Behavior,
    make_agent_with_param,
    messaging::*,
    ErrorCode,
};

#[derive(Debug)]
pub struct CorrectorParams {
    r: kalman::MatrixType,
}

impl CorrectorParams {
    pub fn new_multiple_noise(
        n0: kalman::DataType,
        n1: kalman::DataType,
        n2: kalman::DataType,
    ) -> Self {
        let r = kalman::MatrixType::from_diagonal(&kalman::VectorType::new(n0, n1, n2));
        Self { r }
    }
}

const LOGGER: &str = "AgentLogger";
const SENSOR: &str = "AgentSensor";

make_agent_with_param!(pub Corrector, CorrectorParams);

impl Behavior for Corrector {
    fn setup(&mut self) -> Result<(), ErrorCode> {
        println!("START CORRECTOR AGENT");
        self.agent.add_contact(LOGGER)?;
        self.agent.add_contact(SENSOR)
    }

    fn action(&mut self) -> Result<(), ErrorCode> {
        let param = &mut self.param;

        // get predicted state
        let state_covar_msg = self.agent.receive()?;
        state_covar_msg
            .message_type()
            .is_message_type(&MessageType::Inform)?;
        let state_covar_string = state_covar_msg.content().to_string();
        let state_covar: kalman::StateCovariance =
            serde_json::from_str(state_covar_string.as_str())
                .or(Err(ErrorCode::InvalidContent(state_covar_string)))?;

        // request measurements
        self.agent.send_to(
            SENSOR,
            MessageType::Request,
            Content::Action(ActionType::Other("Send Accelerometer")),
        )?;

        // get measurements
        let z_msg = self.agent.receive()?;
        z_msg.message_type().is_message_type(&MessageType::Inform)?;
        let z_content = z_msg.content().to_string();
        let z: kalman::VectorType = serde_json::from_str(z_content.as_str())
            .or(Err(ErrorCode::InvalidContent(z_content)))?;

        // correct
        let correction = kalman::correct(&state_covar, &z, &param.r)
            .map_err(|e| ErrorCode::Other(e.to_string()))?;

        // get string
        let state_covar_string = correction.to_json().map_err(ErrorCode::Other)?;

        // send state
        self.agent.send_to(
            LOGGER,
            MessageType::Inform,
            Content::Expression(state_covar_string),
        )
    }
}

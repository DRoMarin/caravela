use crate::kalman;
use caravela::{
    agent::{Agent, AgentBuildParam},
    behavior::Behavior,
    make_agent_with_param,
    messaging::*,
    ErrorCode,
};

#[derive(Debug)]
pub struct PredictorParams {
    q: kalman::MatrixType,
}

impl PredictorParams {
    pub fn new(noise: kalman::DataType) -> Self {
        let q = kalman::MatrixType::from_diagonal_element(noise);
        Self { q }
    }
}

const CORRECTOR: &str = "AgentCorrector";
const SENSOR: &str = "AgentSensor";

make_agent_with_param!(pub Predictor, PredictorParams);

impl Behavior for Predictor {
    fn setup(&mut self) -> Result<(), ErrorCode> {
        println!("START PREDICTOR AGENT");
        self.agent.add_contact(CORRECTOR)?;
        self.agent.add_contact(SENSOR)
    }

    fn action(&mut self) -> Result<(), ErrorCode> {
        let param = &mut self.param;

        // get previous estimated state
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
            Content::Action(ActionType::Other("Send Gyroscope")),
        )?;

        // get measurements
        let u_msg = self.agent.receive()?;
        u_msg.message_type().is_message_type(&MessageType::Inform)?;
        let u_content = u_msg.content().to_string();
        let u: kalman::VectorType = serde_json::from_str(u_content.as_str())
            .or(Err(ErrorCode::InvalidContent(u_content)))?;

        // predict
        let prediction = kalman::predict(&state_covar, kalman::DT, &u, &param.q);

        // get string
        let state_covar_string = prediction.to_json().map_err(ErrorCode::Other)?;

        // send state
        self.agent.send_to(
            CORRECTOR,
            MessageType::Inform,
            Content::Expression(state_covar_string),
        )
    }
}

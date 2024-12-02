mod corrector;
mod logger;
mod predictor;
mod sensor;

use caravela::{Platform, DEFAULT_STACK};
use std::error::Error;

use corrector::*;
use logger::*;
use predictor::*;
use sensor::*;

use super::{Q, R0, R1, R2};

pub fn caravela_main(
    in_filepath: &'static str,
    out_filepath: &'static str,
) -> Result<(), Box<dyn Error>> {
    let sensor_params = SensorParams::new(in_filepath)?;

    let logger_params = LoggerParams::new(out_filepath)?;

    let predictor_params = PredictorParams::new(Q);

    let corrector_params = CorrectorParams::new_multiple_noise(R0, R1, R2);

    let agent_platform = Platform::new("adcs")?;
    // add agents
    let sensor = agent_platform.add_agent_with_param::<Sensor>(
        "AgentSensor",
        3,
        DEFAULT_STACK,
        sensor_params,
    )?;
    let logger = agent_platform.add_agent_with_param::<Logger>(
        "AgentLogger",
        1,
        DEFAULT_STACK,
        logger_params,
    )?;
    let predictor = agent_platform.add_agent_with_param::<Predictor>(
        "AgentPredictor",
        2,
        DEFAULT_STACK,
        predictor_params,
    )?;
    let corrector = agent_platform.add_agent_with_param::<Corrector>(
        "AgentCorrector",
        2,
        DEFAULT_STACK,
        corrector_params,
    )?;
    // start agents
    agent_platform.start(&sensor)?;
    agent_platform.start(&logger)?;
    agent_platform.start(&predictor)?;
    agent_platform.start(&corrector)?;

    //std::thread::sleep(std::time::Duration::from_millis(10000));
    std::thread::park();

    Ok(())
}

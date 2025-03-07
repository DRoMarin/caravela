use std::error::Error;

use kalman::DataType;

#[cfg(all(feature = "agents", feature = "threads"))]
compile_error!("choose only one method: agents or threads");

mod imu_data;
mod kalman;
#[cfg(feature = "agents")]
mod kalman_agents;
#[cfg(feature = "threads")]
mod kalman_threads;

// process noise
const Q0: DataType = 2.98116e-7;
const Q1: DataType = 2.98116e-7;
const Q2: DataType = 2.98116e-7;

// observation noise
const R0: DataType = 6.22363e-5;
const R1: DataType = 6.22363e-5;
const R2: DataType = 6.22363e-5;

fn main() -> Result<(), Box<dyn Error>> {

    // Paths relative to the location of the binary, 
    // which will be placed at the top of the repo for ease of use.
    let in_filepath = "examples/kalman/records/measurements.csv";
    let out_filepath = "examples/kalman/records/pred_position.csv";

    #[cfg(feature = "agents")]
    let _ = kalman_agents::caravela_main(in_filepath, out_filepath);
    #[cfg(feature = "threads")]
    let _ = kalman_threads::threads_main(in_filepath, out_filepath);
    Ok(())
}

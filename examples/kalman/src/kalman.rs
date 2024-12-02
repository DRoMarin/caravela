mod kalman_types;

pub use kalman_types::*;
use nalgebra::{matrix, vector};

#[derive(Debug)]
pub enum KalmanFilterError {
    NotSquareMatrix(MatrixType),
}

impl std::fmt::Display for KalmanFilterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            KalmanFilterError::NotSquareMatrix(x) => write!(
                f,
                "Operation could not be completed because matrix is not square: {:?}",
                x
            ),
        }
    }
}

impl std::error::Error for KalmanFilterError {}

pub fn f(x: &VectorType, u: &VectorType) -> VectorType {
    let phi = x[0];
    let theta = x[1];
    //let psi = x[2];
    let s_phi = phi.sin();
    let c_phi = phi.cos();
    let t_theta = theta.tan();
    let c_theta = theta.cos();

    let f = matrix![
       1.0,    s_phi*t_theta,  c_phi*t_theta;
       0.0,    c_phi,         -s_phi;
       0.0,    s_phi/c_theta,  c_phi/c_theta];
    f * u
}

pub fn f_jacobian(x: &VectorType, u: &VectorType) -> MatrixType {
    let phi = x[0];
    let theta = x[1];
    //let psi = x[2];
    //let p = u[0];
    let q = u[1];
    let r = u[2];
    //let t_phi = phi.tan();
    let s_phi = phi.sin();
    let c_phi = phi.cos();
    let t_theta = theta.tan();
    let s_theta = theta.sin();
    let c_theta = theta.cos();

    let c_theta2 = c_theta * c_theta;

    matrix![
        (q*c_phi - r*s_phi)*t_theta,    (q*s_phi + r*c_phi)/c_theta2,           0.0;
        -q*s_phi - r*c_phi,             0.0,                                    0.0;
        (q*c_phi - r*s_phi)/c_theta,    (q*s_phi + r*c_phi)*s_theta/c_theta2,   0.0]
}

pub fn h(x: &VectorType) -> VectorType {
    let phi = x[0];
    let theta = x[1];
    //let t_phi = phi.tan();
    let s_phi = phi.sin();
    let c_phi = phi.cos();
    //let t_theta = theta.tan();
    let s_theta = theta.sin();
    let c_theta = theta.cos();

    G * vector![s_theta, -c_theta * s_phi, -c_theta * c_phi]
}

pub fn h_jacobian(x: &VectorType) -> MatrixType {
    let phi = x[0];
    let theta = x[1];
    //let t_phi = phi.tan();
    let s_phi = phi.sin();
    let c_phi = phi.cos();
    //let t_theta = theta.tan();
    let s_theta = theta.sin();
    let c_theta = theta.cos();

    G * matrix![
         0.0,           c_theta,        0.0; 
        -c_phi*c_theta, s_phi*s_theta,  0.0; 
         s_phi*c_theta, s_theta*c_phi,  0.0]
}

pub fn k(p: &MatrixType, c: &MatrixType, r: &MatrixType) -> Result<MatrixType, KalmanFilterError> {
    let s = c * p * c.transpose() + r;
    let s_inv = s
        .try_inverse()
        .ok_or(KalmanFilterError::NotSquareMatrix(s))?;
    Ok(p * c.transpose() * s_inv)
}

pub fn predict(
    state_covar: &StateCovariance,
    dt: DataType,
    u: &VectorType,
    q: &MatrixType,
) -> StateCovariance {
    let x = state_covar.x() + dt * f(state_covar.x(), u);
    let a = f_jacobian(state_covar.x(), u);
    let p = state_covar.p()
        + kalman_types::DT * (a * state_covar.p() + state_covar.p() * a.transpose() + q);
    StateCovariance::new_from(x, p)
}

pub fn correct(
    state_covar: &StateCovariance,
    z: &VectorType,
    r: &MatrixType,
) -> Result<StateCovariance, KalmanFilterError> {
    let c = h_jacobian(state_covar.x());
    let k = k(state_covar.p(), &c, r)?;
    let h = h(state_covar.x());
    let y = z - h;
    let x = state_covar.x() + k * y;
    let p = (MatrixType::identity() - k * c) * state_covar.p();
    Ok(StateCovariance::new_from(x, p))
}

use nalgebra::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

pub type DataType = f32;
pub const DT: DataType = 0.0005;
pub const G: DataType = 9.81;
pub type VectorType = Vector3<DataType>;
pub type MatrixType = Matrix3<DataType>;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct StateCovariance {
    x: VectorType,
    p: MatrixType,
}

/*impl Display for StateCovariance{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

    }
}*/

impl StateCovariance {
    pub fn new() -> Self {
        let x = VectorType::zeros();
        let p = MatrixType::identity();
        Self { x, p }
    }

    pub fn new_from(x: VectorType, p: MatrixType) -> Self {
        Self { x, p }
    }

    pub fn x(&self) -> &VectorType {
        &self.x
    }

    pub fn p(&self) -> &MatrixType {
        &self.p
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
}

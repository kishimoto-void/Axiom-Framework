use chrono::{DateTime, Utc};
use nalgebra::{Cholesky, DMatrix, DVector};

use crate::error::DCKError;

/// Multi-dimensional state estimate using nalgebra.
#[derive(Debug, Clone)]
pub struct StateEstimate {
    pub mean: DVector<f64>,
    pub covariance: DMatrix<f64>,
    cholesky_l: Option<DMatrix<f64>,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

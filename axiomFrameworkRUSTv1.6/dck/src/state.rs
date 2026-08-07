use chrono::{DateTime, Utc};
use nalgebra::{Cholesky, DMatrix, DVector};

use crate::error::DCKError;

/// Multi-dimensional state estimate using nalgebra.
///
/// - `mean`: DVector
/// - `covariance`: symmetric positive-(semi)definite DMatrix
/// - Precomputed Cholesky factor (L) for fast Mahalanobis distance.
#[derive(Debug, Clone)]
pub struct StateEstimate {
    pub mean: DVector<f64>,
    pub covariance: DMatrix<f64>,
    /// Lower-triangular Cholesky factor L such that L * L^T ≈ covariance.
    cholesky_l: Option<DMatrix<f64>>,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

impl StateEstimate {
    pub fn new(
        mean: DVector<f64>,
        covariance: DMatrix<f64>,
        confidence: f64,
        timestamp: DateTime<Utc>,
        source: impl Into<String>,
        cholesky_floor: f64,
    ) -> Result<Self, DCKError> {
        let dim = mean.len();
        if covariance.nrows() != dim || covariance.ncols() != dim {
            return Err(DCKError::ValidationError(
                "Covariance dimension does not match mean".into(),
            ));
        }

        let mut cov = (&covariance + covariance.transpose()) * 0.5;

        for i in 0..dim {
            if cov[(i, i)] < cholesky_floor {
                cov[(i, i)] = cholesky_floor;
            }
        }

        let cholesky_l = match Cholesky::new(cov.clone()) {
            Some(chol) => Some(chol.l().into_owned()),
            None => {
                for i in 0..dim {
                    cov[(i, i)] += cholesky_floor * 10.0;
                }
                let chol = Cholesky::new(cov.clone()).ok_or_else(|| {
                    DCKError::LinAlg(
                        "Cholesky decomposition failed even after regularization".into(),
                    )
                })?;
                Some(chol.l().into_owned())
            }
        };

        Ok(Self {
            mean,
            covariance: cov,
            cholesky_l,
            confidence: confidence.clamp(0.0, 1.0),
            timestamp,
            source: source.into(),
        })
    }

    pub fn scalar(
        mean: f64,
        variance: f64,
        confidence: f64,
        timestamp: DateTime<Utc>,
        source: impl Into<String>,
        cholesky_floor: f64,
    ) -> Result<Self, DCKError> {
        let m = DVector::from_element(1, mean);
        let c = DMatrix::from_element(1, 1, variance.max(cholesky_floor));
        Self::new(m, c, confidence, timestamp, source, cholesky_floor)
    }

    pub fn dim(&self) -> usize {
        self.mean.len()
    }

    pub fn mahalanobis_distance(&self, target: &DVector<f64>) -> Result<f64, DCKError> {
        if target.len() != self.dim() {
            return Err(DCKError::ValidationError(
                "Target dimension mismatch".into(),
            ));
        }

        let diff = &self.mean - target;

        let Some(ref l) = self.cholesky_l else {
            return Ok(diff.norm());
        };

        let mut y = DVector::zeros(self.dim());
        for i in 0..self.dim() {
            let mut sum = 0.0;
            for j in 0..i {
                sum += l[(i, j)] * y[j];
            }
            let diag = l[(i, i)];
            if diag.abs() < 1e-15 {
                y[i] = 0.0;
            } else {
                y[i] = (diff[i] - sum) / diag;
            }
        }

        Ok(y.norm())
    }

    pub fn mahalanobis_distance_scalar(&self, target: f64) -> f64 {
        if self.dim() == 1 {
            let std = self.covariance[(0, 0)].sqrt().max(1e-12);
            (self.mean[0] - target).abs() / std
        } else {
            (self.mean.mean() - target).abs()
        }
    }

    pub fn total_uncertainty(&self, min_uncertainty: f64) -> f64 {
        if self.dim() == 1 {
            let var = self.covariance[(0, 0)].max(1e-12);
            (0.5 * var.ln()).exp().max(min_uncertainty)
        } else {
            let det = self.covariance.determinant().abs().max(1e-30);
            det.powf(1.0 / self.dim() as f64).max(min_uncertainty)
        }
    }
}

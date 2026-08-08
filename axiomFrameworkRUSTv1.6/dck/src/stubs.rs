use std::collections::HashMap;

use async_trait::async_trait;
use nalgebra::{DMatrix, DVector};

use crate::capabilities::{ExecutorCapability, ObserverCapability, PredictorCapability};
use crate::clock::Clock;
use crate::error::DCKError;
use crate::event::ActionType;
use crate::resource::ResourceVector;
use crate::state::StateEstimate;

pub struct StubObserver;

#[async_trait]
impl ObserverCapability for StubObserver {
    async fn observe(
        &self,
        raw_telemetry: &HashMap<String, f64>,
        clock: &dyn Clock,
        cholesky_floor: f64,
    ) -> Result<HashMap<String, StateEstimate>, DCKError> {
        let mut map = HashMap::new();
        let now = clock.now();
        for (k, &v) in raw_telemetry {
            let est = StateEstimate::scalar(v, 0.25, 0.95, now, "stub_observer", cholesky_floor)?;
            map.insert(k.clone(), est);
        }
        Ok(map)
    }
}

pub struct StubPredictor;

#[async_trait]
impl PredictorCapability for StubPredictor {
    async fn forecast(
        &self,
        _metric_name: &str,
        est: &StateEstimate,
        horizon: u32,
        cholesky_floor: f64,
    ) -> Result<StateEstimate, DCKError> {
        let decay = (1.0 - 0.02 * horizon as f64).max(0.5);
        let new_mean = est.mean.map(|x| x * decay);
        let scale = 1.0 + 0.05 * horizon as f64;
        let new_cov = &est.covariance * scale;
        StateEstimate::new(
            new_mean,
            new_cov,
            (est.confidence - 0.03 * horizon as f64).max(0.1),
            est.timestamp,
            "stub_predictor",
            cholesky_floor,
        )
    }
}

pub struct MultiDimStubObserver;

#[async_trait]
impl ObserverCapability for MultiDimStubObserver {
    async fn observe(
        &self,
        raw_telemetry: &HashMap<String, f64>,
        clock: &dyn Clock,
        cholesky_floor: f64,
    ) -> Result<HashMap<String, StateEstimate>, DCKError> {
        let mut map = HashMap::new();
        let now = clock.now();
        if let (Some(&temp), Some(&hum)) = (
            raw_telemetry.get("temperature"),
            raw_telemetry.get("humidity"),
        ) {
            let mean = DVector::from_vec(vec![temp, hum]);
            let cov = DMatrix::from_row_slice(2, 2, &[0.25, 0.05, 0.05, 0.36]);
            let est = StateEstimate::new(mean, cov, 0.92, now, "multidim_stub", cholesky_floor)?;
            map.insert("climate".into(), est);
        }
        for (k, &v) in raw_telemetry {
            let est = StateEstimate::scalar(v, 0.25, 0.95, now, "stub_observer", cholesky_floor)?;
            map.insert(k.clone(), est);
        }
        Ok(map)
    }
}

pub struct StubExecutor;

#[async_trait]
impl ExecutorCapability for StubExecutor {
    async fn execute(&self, _action: ActionType, _resource: &ResourceVector) -> bool {
        true
    }
}

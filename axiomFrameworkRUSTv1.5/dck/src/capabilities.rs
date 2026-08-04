use std::collections::HashMap;

use async_trait::async_trait;

use crate::clock::Clock;
use crate::error::DCKError;
use crate::event::ActionType;
use crate::resource::ResourceVector;
use crate::state::StateEstimate;

#[async_trait]
pub trait ObserverCapability: Send + Sync {
    async fn observe(
        &self,
        raw_telemetry: &HashMap<String, f64>,
        clock: &dyn Clock,
        cholesky_floor: f64,
    ) -> Result<HashMap<String, StateEstimate>, DCKError>;
}

#[async_trait]
pub trait PredictorCapability: Send + Sync {
    async fn forecast(
        &self,
        metric_name: &str,
        est: &StateEstimate,
        horizon: u32,
        cholesky_floor: f64,
    ) -> Result<StateEstimate, DCKError>;
}

#[async_trait]
pub trait ExecutorCapability: Send + Sync {
    async fn execute(&self, action: ActionType, resource: &ResourceVector) -> bool;
}

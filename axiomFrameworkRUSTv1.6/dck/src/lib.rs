//! Difference Convergence Kernel (DCK) v2.2 (AXIOM Framework Rust v1.6)
//!
//! Fully modular + nalgebra multi-dimensional implementation.
//!
//! v2.2 (measurement library completeness):
//! - DifferenceBreakdown { position, velocity, covariance, confidence }
//! - ConvergenceReason { ThresholdReached, MaxTick, Divergence, NumericalIssue, InProgress }
//! - StabilityScore { score, speed, smoothness, final_accuracy }
//! - ConvergenceReport::history() / difference_curve() / finish()
//!
//! v2.1:
//! - DifferenceMetrics / ConvergenceReport / evaluate_difference
//! - MockClock / Golden unit tests
//!
//! Design goals retained:
//! - Module separation, nalgebra multi-D, injected Clock, config-driven scoring

pub mod clock;
pub mod ids;
pub mod error;
pub mod config;
pub mod state;
pub mod resource;
pub mod lease;
pub mod capabilities;
pub mod intent;
pub mod event;
pub mod gap;
pub mod kernel;
pub mod stubs;
pub mod metrics;
pub mod golden_tests;

// Re-exports
pub use clock::{Clock, MockClock, SystemClock};
pub use ids::{EventId, IntentId, KernelId, LeaseId};
pub use error::DCKError;
pub use config::DCKConfig;
pub use state::StateEstimate;
pub use resource::{IrreversibleResource, ResourceVector, ReversibleResource};
pub use lease::{LeaseManager, LeaseRecord, LeaseState};
pub use capabilities::{ExecutorCapability, ObserverCapability, PredictorCapability};
pub use intent::{Intent, IntentRecord, IntentScheduler, MetricGoal};
pub use event::{ActionType, TransitionEvent, TransitionStage};
pub use gap::GapHistory;
pub use kernel::{DifferenceConvergenceKernel, KernelBuilder};
pub use stubs::{StubExecutor, StubObserver, StubPredictor};
pub use metrics::{
    evaluate_difference, evaluate_difference_with_velocity,
    ConvergenceReason, ConvergenceReport, DifferenceBreakdown, DifferenceMetrics,
    StabilityScore,
};

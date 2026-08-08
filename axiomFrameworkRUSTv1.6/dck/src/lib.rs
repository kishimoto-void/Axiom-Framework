//! Difference Convergence Kernel (DCK) v2.3 (AXIOM Framework Rust v1.6)
//!
//! Fully modular + nalgebra multi-dimensional implementation.
//!
//! v2.3 (dual-hash difference taxonomy):
//! - HashA (Invariant/Physical, ACP Ground Truth) / HashB (Semantic)
//! - DualHashClass matrix: None | Semantic | State | Compound
//! - ConstraintVerdict + DifferenceKind (State / Semantic / Constraint)
//! - DualHashEvaluation for DCK / CI / hallucination detection
//!
//! v2.2 (measurement library completeness):
//! - DifferenceBreakdown { position, velocity, covariance, confidence }
//! - ConvergenceReason / StabilityScore / ConvergenceReport
//!
//! v2.1:
//! - DifferenceMetrics / evaluate_difference / MockClock
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
pub mod dual_hash;
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
pub use dual_hash::{
    validate_constraint, ConstraintVerdict, DifferenceKind, DualHashClass,
    DualHashEvaluation, DualHashSnapshot, HashA, HashB,
};

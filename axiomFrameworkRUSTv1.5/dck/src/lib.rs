//! Difference Convergence Kernel (DCK) v2.0
//! Fully modular + nalgebra multi-dimensional implementation.
//!
//! Design goals addressed from review:
//! - Proper module separation
//! - Real multi-dimensional state with nalgebra (DVector / DMatrix + Cholesky)
//! - Strict Newtypes (no blanket From)
//! - Clean locking & join_all concurrency
//! - Injected Clock
//! - Config parameters actually used in scoring

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

// Re-exports for convenience
pub use clock::{Clock, SystemClock};
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

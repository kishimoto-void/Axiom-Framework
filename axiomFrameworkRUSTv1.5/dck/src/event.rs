use chrono::{DateTime, Utc};

use crate::ids::{EventId, IntentId, LeaseId};
use crate::state::StateEstimate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    NoAction,
    ExecuteConvergence,
    SafetyHalt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionStage {
    Projected,
    Executed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct TransitionEvent {
    pub event_id: EventId,
    pub intent_id: IntentId,
    pub metric_name: String,
    pub expected_state: f64,
    pub observed_state: StateEstimate,
    pub projected_state: StateEstimate,
    pub created_turn: u64,
    pub created_at: DateTime<Utc>,
    pub lease_id: Option<LeaseId>,
    pub decision_action: Option<ActionType>,
    pub current_stage: TransitionStage,
    pub computed_velocity: f64,
    pub equivalence_gap: f64,
}

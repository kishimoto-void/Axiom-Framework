//! LRP (Reasoning Transition Protocol) v1.5.0-research-rust-strict-final
//!
//! 100% Deterministic Reasoning Session Engine for research reproducibility.
//! - Binary / string level identical results for same seed
//! - Observer Isolation via std::panic::catch_unwind
//! - No wall-clock time, no Uuid::v4, no HashMap (BTreeMap only)
//! - Strict DeltaAction × DeltaKind matching (Removed is kind-isolated)
//! - Full PLP-compatible delta application + Replay / Fork
//!
//! Philosophy: History is Truth / Current is Cache | Quantitative Validation First

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

pub const VERSION: &str = "1.5.0-research-rust-strict-final";

// =============================================================================
// Errors (Deterministic)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LrpError {
    InvalidStateTransition(String),
    ReplayOutOfBounds { requested: usize, available: usize },
    SerializationError(String),
    StateRebuildError(String),
    LockError(String),
    ObserverError(String),
    ClockError(String),
    IdGenerationError(String),
}

impl std::fmt::Display for LrpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidStateTransition(msg) => write!(f, "Invalid State Transition: {}", msg),
            Self::ReplayOutOfBounds { requested, available } => write!(
                f,
                "Replay index out of bounds: requested {}, available {}",
                requested, available
            ),
            Self::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            Self::StateRebuildError(msg) => write!(f, "State Rebuild Error: {}", msg),
            Self::LockError(msg) => write!(f, "Lock Poisoned Error: {}", msg),
            Self::ObserverError(msg) => write!(f, "Observer Isolation Failure: {}", msg),
            Self::ClockError(msg) => write!(f, "Clock Error: {}", msg),
            Self::IdGenerationError(msg) => write!(f, "Id Generation Error: {}", msg),
        }
    }
}

impl std::error::Error for LrpError {}

// =============================================================================
// Philosophy
// =============================================================================

pub const PHILOSOPHY: &[&str] = &[
    "Intelligence Neutral",
    "Model Neutral",
    "Language Neutral",
    "Reasoning Transition First",
    "Observer Isolation",
    "Deterministic Replay",
    "Explainability First",
    "Capability Isolation",
    "Contract Driven",
    "History is Truth / Current is Cache",
    "Quantitative Validation First",
];

// =============================================================================
// 1. Reasoning Primitives
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ReasoningPrimitive {
    Observe,
    Hypothesis,
    Inference,
    Validation,
    Commit,
    Fork,
    Merge,
    Rollback,
}

impl ReasoningPrimitive {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Observe => "Observe",
            Self::Hypothesis => "Hypothesis",
            Self::Inference => "Inference",
            Self::Validation => "Validation",
            Self::Commit => "Commit",
            Self::Fork => "Fork",
            Self::Merge => "Merge",
            Self::Rollback => "Rollback",
        }
    }
}

// =============================================================================
// 2. Determinism Helpers (Strict Sequential, No Wall-Clock Fallback)
// =============================================================================

#[derive(Clone)]
pub struct DeterministicClock {
    current: Arc<Mutex<DateTime<Utc>>>,
    step_seconds: i64,
    /// Fixed anchor used only as last-resort sentinel (never real Utc::now())
    sentinel: DateTime<Utc>,
}

impl DeterministicClock {
    pub fn new(start: Option<DateTime<Utc>>, step_seconds: u64) -> Self {
        let base = start.unwrap_or_else(|| Utc.with_ymd_and_hms(2026, 7, 31, 12, 0, 0).unwrap());
        Self {
            current: Arc::new(Mutex::new(base)),
            step_seconds: step_seconds as i64,
            sentinel: base,
        }
    }

    /// Advances and returns current logical time. Never falls back to wall clock.
    pub fn now(&self) -> Result<DateTime<Utc>, LrpError> {
        let mut guard = self
            .current
            .lock()
            .map_err(|e| LrpError::ClockError(format!("mutex poisoned: {}", e)))?;
        let ret = *guard;
        *guard = *guard + chrono::Duration::seconds(self.step_seconds);
        Ok(ret)
    }

    /// Deterministic sentinel (initial anchor). Used only when explicit recovery is desired.
    pub fn sentinel(&self) -> DateTime<Utc> {
        self.sentinel
    }
}

#[derive(Clone)]
pub struct DeterministicIDFactory {
    seed: usize,
    prefix: String,
    counter: Arc<Mutex<usize>>,
}

impl DeterministicIDFactory {
    pub fn new(seed: usize, prefix: &str) -> Self {
        Self {
            seed,
            prefix: prefix.to_string(),
            counter: Arc::new(Mutex::new(0)),
        }
    }

    pub fn generate(&self, kind: &str) -> Result<String, LrpError> {
        let mut c = self
            .counter
            .lock()
            .map_err(|e| LrpError::IdGenerationError(format!("mutex poisoned: {}", e)))?;
        *c += 1;
        let raw = format!("{}:{}:{}:{}", self.prefix, self.seed, kind, *c);
        let mut hasher = Sha256::new();
        hasher.update(raw.as_bytes());
        let hex = format!("{:x}", hasher.finalize());
        Ok(format!("{}_{}_{}", kind, self.seed, &hex[..12]))
    }
}

// =============================================================================
// 3. Evidence (Order-Preserving Canonical Hash)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceKind {
    Fact,
    Inference,
    Assumption,
    Retrieved,
    Calculated,
    Observed,
    ToolResult,
    Memory,
    Graph,
    Tensor,
    Image,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_id: String,
    pub kind: EvidenceKind,
    pub payload: serde_json::Value,
    pub source: String,
    pub confidence: f64,
    pub derived_from: Vec<String>,
    pub metadata: BTreeMap<String, serde_json::Value>,
    pub content_hash: String,
    pub timestamp: DateTime<Utc>,
}

impl Evidence {
    pub fn new(
        evidence_id: String,
        kind: EvidenceKind,
        payload: serde_json::Value,
        source: String,
        confidence: f64,
        derived_from: Vec<String>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        let payload_str = serde_json::to_string(&payload).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(payload_str.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        Self {
            evidence_id,
            kind,
            payload,
            source,
            confidence: confidence.clamp(0.0, 1.0),
            derived_from,
            metadata: BTreeMap::new(),
            content_hash: hash[..16].to_string(),
            timestamp,
        }
    }
}

// =============================================================================
// 4. Context Graph (Vec for insertion order stability)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextNode {
    pub node_id: String,
    pub kind: String,
    pub payload: serde_json::Value,
    pub metadata: BTreeMap<String, serde_json::Value>,
    pub parent_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContextGraph {
    pub nodes: Vec<ContextNode>,
}

impl ContextGraph {
    pub fn add_or_update(&self, node: ContextNode) -> Self {
        let mut new_nodes = self.nodes.clone();
        if let Some(pos) = new_nodes.iter().position(|n| n.node_id == node.node_id) {
            new_nodes[pos] = node;
        } else {
            new_nodes.push(node);
        }
        Self { nodes: new_nodes }
    }

    pub fn remove(&self, node_id: &str) -> Self {
        let new_nodes = self
            .nodes
            .iter()
            .filter(|n| n.node_id != node_id)
            .cloned()
            .collect();
        Self { nodes: new_nodes }
    }
}

// =============================================================================
// 5. Capability / Contract / Candidate
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    pub capability_id: String,
    pub version: String,
    pub constraints: BTreeMap<String, serde_json::Value>,
    pub metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    pub protocol_id: String,
    pub description: String,
    pub side_effect_free: bool,
    pub required_capabilities: Vec<String>,
    pub input_schema_hint: String,
    pub output_schema_hint: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub candidate_id: String,
    pub description: String,
    pub score: f64,
    pub confidence: f64,
    pub risk: f64,
    pub supporting_evidence_ids: Vec<String>,
    pub metadata: BTreeMap<String, serde_json::Value>,
}

// =============================================================================
// 6. Typed Delta & Action (Strict Kind Isolation)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaAction {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaKind {
    ParticleChange,
    EdgeChange,
    CapabilityChange,
    MemoryChange,
    ContextChange,
    EvidenceChange,
    CandidateChange,
    PrimitiveChange,
    MetricChange,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypedPayload {
    Context(ContextNode),
    Evidence(Evidence),
    Candidate(Candidate),
    Primitive { primitive: String, operation: String },
    Raw(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PLPDelta {
    pub action: DeltaAction,
    pub kind: DeltaKind,
    pub payload: TypedPayload,
    pub target_id: Option<String>,
    pub metadata: BTreeMap<String, serde_json::Value>,
}

// =============================================================================
// 7. Transition
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningTransition {
    pub transition_id: String,
    pub primitive: ReasoningPrimitive,
    pub before_state_id: String,
    pub after_state_id: String,
    pub operation: String,
    pub deltas: Vec<PLPDelta>,
    pub produced_evidence_ids: Vec<String>,
    pub produced_candidate_ids: Vec<String>,
    pub contract_protocol_id: Option<String>,
    pub parent_transition_id: Option<String>,
    pub validation_passed: bool,
    pub validation_message: String,
    pub experiment_id: String,
    pub condition: String,
    pub tags: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: BTreeMap<String, serde_json::Value>,
}

// =============================================================================
// 8. State / Session
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningState {
    pub state_id: String,
    pub context: ContextGraph,
    pub evidence: Vec<Evidence>,
    pub candidates: Vec<Candidate>,
    pub last_primitive: Option<ReasoningPrimitive>,
    pub step_count: usize,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObserverRecord {
    pub record_id: String,
    pub transition_id: String,
    pub protocol_id: String,
    pub metrics: BTreeMap<String, f64>,
    pub timing_ms: f64,
    pub resource_hint: BTreeMap<String, serde_json::Value>,
    pub reason: String,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningSession {
    pub session_id: String,
    pub problem_id: String,
    pub initial_state: ReasoningState,
    pub current_cache: Option<ReasoningState>,
    pub transitions: Vec<ReasoningTransition>,
    pub observer_records: Vec<ObserverRecord>,
    pub capabilities: Vec<Capability>,
    pub contracts: Vec<Contract>,
    pub experiment_id: String,
    pub seed: usize,
    pub condition: String,
    pub created_at: DateTime<Utc>,
    pub version: String,
}

impl ReasoningSession {
    pub fn current_state(&self) -> &ReasoningState {
        self.current_cache.as_ref().unwrap_or(&self.initial_state)
    }

    pub fn with_cache(&self, state: Option<ReasoningState>) -> Self {
        let mut copy = self.clone();
        copy.current_cache = state;
        copy
    }
}

// =============================================================================
// 9. Quantitative Metrics (Deterministic Order via BTreeMap)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub n_transitions: usize,
    pub n_evidence: usize,
    pub n_candidates: usize,
    pub n_validations: usize,
    pub n_validation_pass: usize,
    pub validation_pass_rate: f64,
    pub primitive_counts: BTreeMap<String, usize>,
    pub evidence_growth: Vec<usize>,
    pub mean_evidence_confidence: f64,
    pub mean_candidate_confidence: f64,
    pub n_observer_records: usize,
    pub condition: String,
    pub seed: usize,
    pub experiment_id: String,
}

pub fn compute_session_metrics(session: &ReasoningSession) -> SessionMetrics {
    let mut prim_counts = BTreeMap::new();
    let mut n_val = 0;
    let mut n_val_pass = 0;
    let mut growth = Vec::new();
    let mut ev_count = 0;

    for tr in &session.transitions {
        *prim_counts
            .entry(tr.primitive.as_str().to_string())
            .or_insert(0) += 1;
        if tr.primitive == ReasoningPrimitive::Validation {
            n_val += 1;
            if tr.validation_passed {
                n_val_pass += 1;
            }
        }
        ev_count += tr.produced_evidence_ids.len();
        growth.push(ev_count);
    }

    let current = session.current_state();
    let mean_ev_conf = if !current.evidence.is_empty() {
        current.evidence.iter().map(|e| e.confidence).sum::<f64>() / current.evidence.len() as f64
    } else {
        0.0
    };

    let mean_cand_conf = if !current.candidates.is_empty() {
        current.candidates.iter().map(|c| c.confidence).sum::<f64>() / current.candidates.len() as f64
    } else {
        0.0
    };

    let pass_rate = if n_val > 0 {
        n_val_pass as f64 / n_val as f64
    } else {
        0.0
    };

    SessionMetrics {
        n_transitions: session.transitions.len(),
        n_evidence: current.evidence.len(),
        n_candidates: current.candidates.len(),
        n_validations: n_val,
        n_validation_pass: n_val_pass,
        validation_pass_rate: pass_rate,
        primitive_counts: prim_counts,
        evidence_growth: growth,
        mean_evidence_confidence: mean_ev_conf,
        mean_candidate_confidence: mean_cand_conf,
        n_observer_records: session.observer_records.len(),
        condition: session.condition.clone(),
        seed: session.seed,
        experiment_id: session.experiment_id.clone(),
    }
}

pub fn paper_summary(session: &ReasoningSession) -> String {
    let m = compute_session_metrics(session);
    format!(
        "[LRP Session Metrics] experiment={} condition={} seed={}\n  transitions : {}\n  evidence    : {} (mean conf={:.3})\n  candidates  : {} (mean conf={:.3})\n  validations : {} pass_rate={:.3}\n  primitive_counts : {:?}\n  evidence_growth  : {:?}\n  observer_records : {}",
        if m.experiment_id.is_empty() { "-" } else { &m.experiment_id },
        if m.condition.is_empty() { "-" } else { &m.condition },
        m.seed,
        m.n_transitions,
        m.n_evidence,
        m.mean_evidence_confidence,
        m.n_candidates,
        m.mean_candidate_confidence,
        m.n_validations,
        m.validation_pass_rate,
        m.primitive_counts,
        m.evidence_growth,
        m.n_observer_records
    )
}

// =============================================================================
// 10. Observers (Strict Sequential + catch_unwind Isolation)
// =============================================================================

pub trait IObserver: Send + Sync {
    fn protocol_id(&self) -> &'static str;
    fn observe(
        &self,
        transition: &ReasoningTransition,
        state: &ReasoningState,
        clock: &DeterministicClock,
        id_factory: &DeterministicIDFactory,
    ) -> Result<ObserverRecord, LrpError>;
}

pub struct LatencyObserver;

impl IObserver for LatencyObserver {
    fn protocol_id(&self) -> &'static str {
        "observer.latency.v1"
    }

    fn observe(
        &self,
        transition: &ReasoningTransition,
        state: &ReasoningState,
        clock: &DeterministicClock,
        id_factory: &DeterministicIDFactory,
    ) -> Result<ObserverRecord, LrpError> {
        let mut metrics = BTreeMap::new();
        metrics.insert("step_count".to_string(), state.step_count as f64);

        Ok(ObserverRecord {
            record_id: id_factory.generate("obs")?,
            transition_id: transition.transition_id.clone(),
            protocol_id: self.protocol_id().to_string(),
            metrics,
            timing_ms: 0.0,
            resource_hint: BTreeMap::new(),
            reason: "latency stub".to_string(),
            error: None,
            timestamp: clock.now()?,
        })
    }
}

pub struct MetricObserver;

impl IObserver for MetricObserver {
    fn protocol_id(&self) -> &'static str {
        "observer.metric.v1"
    }

    fn observe(
        &self,
        transition: &ReasoningTransition,
        state: &ReasoningState,
        clock: &DeterministicClock,
        id_factory: &DeterministicIDFactory,
    ) -> Result<ObserverRecord, LrpError> {
        let mut metrics = BTreeMap::new();
        metrics.insert("n_evidence".to_string(), state.evidence.len() as f64);
        metrics.insert("n_candidates".to_string(), state.candidates.len() as f64);
        metrics.insert(
            "validation_passed".to_string(),
            if transition.validation_passed { 1.0 } else { 0.0 },
        );

        Ok(ObserverRecord {
            record_id: id_factory.generate("obs")?,
            transition_id: transition.transition_id.clone(),
            protocol_id: self.protocol_id().to_string(),
            metrics,
            timing_ms: 0.0,
            resource_hint: BTreeMap::new(),
            reason: format!("{}:{}", transition.primitive.as_str(), transition.operation),
            error: None,
            timestamp: clock.now()?,
        })
    }
}

/// Intentional panic observer for isolation testing
pub struct PanickingObserver;

impl IObserver for PanickingObserver {
    fn protocol_id(&self) -> &'static str {
        "observer.panic.test"
    }

    fn observe(
        &self,
        _transition: &ReasoningTransition,
        _state: &ReasoningState,
        _clock: &DeterministicClock,
        _id_factory: &DeterministicIDFactory,
    ) -> Result<ObserverRecord, LrpError> {
        panic!("Intentional Observer Panic for Isolation Test!");
    }
}

pub struct ObserverManager {
    observers: Vec<Arc<dyn IObserver>>,
}

impl ObserverManager {
    pub fn new(observers: Option<Vec<Arc<dyn IObserver>>>) -> Self {
        let default_obs: Vec<Arc<dyn IObserver>> = vec![
            Arc::new(LatencyObserver),
            Arc::new(MetricObserver),
        ];
        Self {
            observers: observers.unwrap_or(default_obs),
        }
    }

    /// Deterministic sequential notification with full panic isolation.
    /// Any panic inside an observer is captured and recorded as error record.
    /// Never propagates panic to the runtime. Never uses wall-clock time.
    pub fn notify(
        &self,
        session: &ReasoningSession,
        transition: &ReasoningTransition,
        clock: &DeterministicClock,
        id_factory: &DeterministicIDFactory,
    ) -> ReasoningSession {
        let state = session.current_state().clone();
        let mut records = Vec::new();

        for obs in &self.observers {
            let protocol = obs.protocol_id().to_string();
            let obs_clone = Arc::clone(obs);
            let tr_clone = transition.clone();
            let state_clone = state.clone();
            let clock_clone = clock.clone();
            let id_clone = id_factory.clone();

            // Full isolation via catch_unwind
            let result = catch_unwind(AssertUnwindSafe(|| {
                obs_clone.observe(&tr_clone, &state_clone, &clock_clone, &id_clone)
            }));

            match result {
                Ok(Ok(record)) => records.push(record),
                Ok(Err(err)) => {
                    // Soft error from observer logic
                    let rec_id = id_factory.generate("obs_err").unwrap_or_else(|_| {
                        format!("obs_err_fallback_{}", records.len())
                    });
                    // Use sentinel only if clock itself fails (extremely rare)
                    let ts = clock.now().unwrap_or_else(|_| clock.sentinel());
                    records.push(ObserverRecord {
                        record_id: rec_id,
                        transition_id: transition.transition_id.clone(),
                        protocol_id: protocol,
                        metrics: BTreeMap::new(),
                        timing_ms: 0.0,
                        resource_hint: BTreeMap::new(),
                        reason: "Observer Execution Failed".to_string(),
                        error: Some(err.to_string()),
                        timestamp: ts,
                    });
                }
                Err(_panic_payload) => {
                    // Hard panic captured
                    let rec_id = id_factory.generate("obs_err").unwrap_or_else(|_| {
                        format!("obs_err_panic_{}", records.len())
                    });
                    let ts = clock.now().unwrap_or_else(|_| clock.sentinel());
                    records.push(ObserverRecord {
                        record_id: rec_id,
                        transition_id: transition.transition_id.clone(),
                        protocol_id: protocol,
                        metrics: BTreeMap::new(),
                        timing_ms: 0.0,
                        resource_hint: BTreeMap::new(),
                        reason: "Observer panicked (captured deterministically)".to_string(),
                        error: Some("Observer panicked (captured deterministically)".to_string()),
                        timestamp: ts,
                    });
                }
            }
        }

        let mut updated = session.clone();
        updated.observer_records.extend(records);
        updated
    }
}

// =============================================================================
// 11. Replay Engine (Strict DeltaAction × DeltaKind Application)
// =============================================================================

pub struct ReplayEngine;

impl ReplayEngine {
    pub fn replay(
        &self,
        session: &ReasoningSession,
        up_to: Option<usize>,
    ) -> Result<ReasoningSession, LrpError> {
        if session.transitions.is_empty() {
            return Ok(session.clone());
        }

        let target_len = match up_to {
            Some(idx) => {
                if idx >= session.transitions.len() {
                    return Err(LrpError::ReplayOutOfBounds {
                        requested: idx,
                        available: session.transitions.len(),
                    });
                }
                idx + 1
            }
            None => session.transitions.len(),
        };

        let truncated_transitions = session.transitions[..target_len].to_vec();
        let valid_tr_ids: BTreeSet<String> = truncated_transitions
            .iter()
            .map(|t| t.transition_id.clone())
            .collect();

        let replayed_state = Self::rebuild_state(&session.initial_state, &truncated_transitions)?;

        // Keep only observer records that belong to surviving transitions
        let filtered_records = session
            .observer_records
            .iter()
            .filter(|r| valid_tr_ids.contains(&r.transition_id))
            .cloned()
            .collect();

        let mut replayed_session = session.clone();
        replayed_session.transitions = truncated_transitions;
        replayed_session.observer_records = filtered_records;
        Ok(replayed_session.with_cache(Some(replayed_state)))
    }

    pub fn fork(
        &self,
        session: &ReasoningSession,
        at: usize,
        new_condition: &str,
        id_factory: &DeterministicIDFactory,
    ) -> Result<ReasoningSession, LrpError> {
        if session.transitions.is_empty() || at >= session.transitions.len() {
            return Err(LrpError::ReplayOutOfBounds {
                requested: at,
                available: session.transitions.len(),
            });
        }

        let truncated = session.transitions[..=at].to_vec();
        let valid_tr_ids: BTreeSet<String> = truncated.iter().map(|t| t.transition_id.clone()).collect();
        let forked_state = Self::rebuild_state(&session.initial_state, &truncated)?;

        let filtered_records = session
            .observer_records
            .iter()
            .filter(|r| valid_tr_ids.contains(&r.transition_id))
            .cloned()
            .collect();

        let mut forked = session.clone();
        forked.session_id = id_factory.generate("fork")?;
        forked.transitions = truncated;
        forked.observer_records = filtered_records;
        forked.condition = if new_condition.is_empty() {
            format!("{}+fork@{}", session.condition, at)
        } else {
            new_condition.to_string()
        };

        Ok(forked.with_cache(Some(forked_state)))
    }

    pub fn rebuild_state(
        initial: &ReasoningState,
        transitions: &[ReasoningTransition],
    ) -> Result<ReasoningState, LrpError> {
        let mut curr = initial.clone();
        for tr in transitions {
            curr = Self::apply_transition(&curr, tr)?;
        }
        Ok(curr)
    }

    /// Strict application: Removed is isolated by DeltaKind.
    /// ContextRemove only touches context, EvidenceRemove only evidence, etc.
    pub fn apply_transition(
        current: &ReasoningState,
        tr: &ReasoningTransition,
    ) -> Result<ReasoningState, LrpError> {
        let mut next = current.clone();

        for delta in &tr.deltas {
            match (&delta.action, &delta.kind, &delta.payload) {
                // --- Context ---
                (DeltaAction::Added | DeltaAction::Modified, DeltaKind::ContextChange, TypedPayload::Context(node)) => {
                    next.context = next.context.add_or_update(node.clone());
                }
                (DeltaAction::Removed, DeltaKind::ContextChange, _) => {
                    if let Some(target_id) = &delta.target_id {
                        next.context = next.context.remove(target_id);
                    }
                }

                // --- Evidence ---
                (DeltaAction::Added, DeltaKind::EvidenceChange, TypedPayload::Evidence(ev)) => {
                    if !next.evidence.iter().any(|e| e.evidence_id == ev.evidence_id) {
                        next.evidence.push(ev.clone());
                    }
                }
                (DeltaAction::Modified, DeltaKind::EvidenceChange, TypedPayload::Evidence(ev)) => {
                    if let Some(pos) = next.evidence.iter().position(|e| e.evidence_id == ev.evidence_id) {
                        next.evidence[pos] = ev.clone();
                    } else {
                        next.evidence.push(ev.clone());
                    }
                }
                (DeltaAction::Removed, DeltaKind::EvidenceChange, _) => {
                    if let Some(target_id) = &delta.target_id {
                        next.evidence.retain(|e| &e.evidence_id != target_id);
                    }
                }

                // --- Candidate ---
                (DeltaAction::Added, DeltaKind::CandidateChange, TypedPayload::Candidate(cand)) => {
                    if !next.candidates.iter().any(|c| c.candidate_id == cand.candidate_id) {
                        next.candidates.push(cand.clone());
                    }
                }
                (DeltaAction::Modified, DeltaKind::CandidateChange, TypedPayload::Candidate(cand)) => {
                    if let Some(pos) = next.candidates.iter().position(|c| c.candidate_id == cand.candidate_id)
                    {
                        next.candidates[pos] = cand.clone();
                    } else {
                        next.candidates.push(cand.clone());
                    }
                }
                (DeltaAction::Removed, DeltaKind::CandidateChange, _) => {
                    if let Some(target_id) = &delta.target_id {
                        next.candidates.retain(|c| &c.candidate_id != target_id);
                    }
                }

                // Primitive / others are recorded in transition only
                _ => {}
            }
        }

        next.state_id = tr.after_state_id.clone();
        next.last_primitive = Some(tr.primitive.clone());
        next.step_count += 1;
        next.notes = tr.operation.clone();
        next.created_at = tr.timestamp;
        Ok(next)
    }
}

// =============================================================================
// 12. LRPRuntime (Sync, Strict Deterministic)
// =============================================================================

pub struct LRPRuntime {
    pub seed: usize,
    pub id_factory: DeterministicIDFactory,
    pub clock: DeterministicClock,
    pub observer_manager: ObserverManager,
    pub replay_engine: ReplayEngine,
}

impl LRPRuntime {
    pub fn new(seed: usize, clock: Option<DeterministicClock>) -> Self {
        Self {
            seed,
            id_factory: DeterministicIDFactory::new(seed, "lrp"),
            clock: clock.unwrap_or_else(|| DeterministicClock::new(None, 1)),
            observer_manager: ObserverManager::new(None),
            replay_engine: ReplayEngine,
        }
    }

    pub fn with_observers(mut self, observers: Vec<Arc<dyn IObserver>>) -> Self {
        self.observer_manager = ObserverManager::new(Some(observers));
        self
    }

    pub fn create_session(
        &self,
        problem_id: &str,
        capabilities: Vec<Capability>,
        contracts: Vec<Contract>,
        initial_context: Option<ContextGraph>,
        experiment_id: &str,
        condition: &str,
    ) -> Result<ReasoningSession, LrpError> {
        let sid = self.id_factory.generate("state")?;
        let now = self.clock.now()?;

        let initial = ReasoningState {
            state_id: sid,
            context: initial_context.unwrap_or_default(),
            evidence: vec![],
            candidates: vec![],
            last_primitive: None,
            step_count: 0,
            notes: "".to_string(),
            created_at: now,
        };

        Ok(ReasoningSession {
            session_id: self.id_factory.generate("session")?,
            problem_id: problem_id.to_string(),
            initial_state: initial.clone(),
            current_cache: Some(initial),
            transitions: vec![],
            observer_records: vec![],
            capabilities,
            contracts,
            experiment_id: if experiment_id.is_empty() {
                format!("exp_{}", self.seed)
            } else {
                experiment_id.to_string()
            },
            seed: self.seed,
            condition: condition.to_string(),
            created_at: now,
            version: VERSION.to_string(),
        })
    }

    pub fn transition(
        &self,
        session: &ReasoningSession,
        primitive: ReasoningPrimitive,
        operation: &str,
        context_updates: Vec<ContextNode>,
        new_evidence: Vec<Evidence>,
        new_candidates: Vec<Candidate>,
        deltas: Vec<PLPDelta>,
        validation_passed: bool,
        validation_message: &str,
        tags: Vec<String>,
    ) -> Result<ReasoningSession, LrpError> {
        let prev = session.current_state();
        let after_state_id = self.id_factory.generate("state")?;
        let now = self.clock.now()?;

        let mut auto_deltas = deltas;

        for node in &context_updates {
            auto_deltas.push(PLPDelta {
                action: DeltaAction::Added,
                kind: DeltaKind::ContextChange,
                payload: TypedPayload::Context(node.clone()),
                target_id: Some(node.node_id.clone()),
                metadata: BTreeMap::new(),
            });
        }

        for ev in &new_evidence {
            auto_deltas.push(PLPDelta {
                action: DeltaAction::Added,
                kind: DeltaKind::EvidenceChange,
                payload: TypedPayload::Evidence(ev.clone()),
                target_id: Some(ev.evidence_id.clone()),
                metadata: BTreeMap::new(),
            });
        }

        for cand in &new_candidates {
            auto_deltas.push(PLPDelta {
                action: DeltaAction::Added,
                kind: DeltaKind::CandidateChange,
                payload: TypedPayload::Candidate(cand.clone()),
                target_id: Some(cand.candidate_id.clone()),
                metadata: BTreeMap::new(),
            });
        }

        auto_deltas.push(PLPDelta {
            action: DeltaAction::Added,
            kind: DeltaKind::PrimitiveChange,
            payload: TypedPayload::Primitive {
                primitive: primitive.as_str().to_string(),
                operation: operation.to_string(),
            },
            target_id: None,
            metadata: BTreeMap::new(),
        });

        let transition = ReasoningTransition {
            transition_id: self.id_factory.generate("tr")?,
            primitive,
            before_state_id: prev.state_id.clone(),
            after_state_id,
            operation: operation.to_string(),
            deltas: auto_deltas,
            produced_evidence_ids: new_evidence.iter().map(|e| e.evidence_id.clone()).collect(),
            produced_candidate_ids: new_candidates.iter().map(|c| c.candidate_id.clone()).collect(),
            contract_protocol_id: None,
            parent_transition_id: None,
            validation_passed,
            validation_message: validation_message.to_string(),
            experiment_id: session.experiment_id.clone(),
            condition: session.condition.clone(),
            tags,
            timestamp: now,
            metadata: BTreeMap::new(),
        };

        let after_state = ReplayEngine::apply_transition(prev, &transition)?;

        let mut updated_session = session.clone();
        updated_session.transitions.push(transition.clone());
        let updated_session = updated_session.with_cache(Some(after_state));

        Ok(self
            .observer_manager
            .notify(&updated_session, &transition, &self.clock, &self.id_factory))
    }
}

// =============================================================================
// Demo Runner (used by tests and binary)
// =============================================================================

pub fn run_condition(seed: usize, condition: &str) -> Result<ReasoningSession, LrpError> {
    let runtime = LRPRuntime::new(seed, None);

    let ctx_node = ContextNode {
        node_id: runtime.id_factory.generate("ctx")?,
        kind: "problem".to_string(),
        payload: serde_json::json!({ "goal": "reduce temperature gap", "target": 25.0 }),
        metadata: BTreeMap::new(),
        parent_ids: vec![],
    };

    let ctx = ContextGraph::default().add_or_update(ctx_node);
    let mut session = runtime.create_session(
        "temp_gap",
        vec![],
        vec![],
        Some(ctx),
        "lrp_v15_demo",
        condition,
    )?;

    // 1. OBSERVE
    let ev_obs = Evidence::new(
        runtime.id_factory.generate("ev")?,
        EvidenceKind::Observed,
        serde_json::json!({ "temperature": 29.2 }),
        "sensor".to_string(),
        0.95,
        vec![],
        runtime.clock.now()?,
    );

    session = runtime.transition(
        &session,
        ReasoningPrimitive::Observe,
        "read temperature",
        vec![],
        vec![ev_obs.clone()],
        vec![],
        vec![],
        true,
        "",
        vec!["sensor".to_string()],
    )?;

    // 2. HYPOTHESIS
    session = runtime.transition(
        &session,
        ReasoningPrimitive::Hypothesis,
        "positive gap decreasing",
        vec![],
        vec![],
        vec![],
        vec![],
        true,
        "",
        vec!["hypothesis".to_string()],
    )?;

    // 3. INFERENCE
    let conf = if condition != "ablation_low_conf" { 0.8 } else { 0.4 };
    let ev_inf = Evidence::new(
        runtime.id_factory.generate("ev")?,
        EvidenceKind::Inference,
        serde_json::json!({ "projected": 27.6 }),
        "model".to_string(),
        conf,
        vec![ev_obs.evidence_id.clone()],
        runtime.clock.now()?,
    );

    session = runtime.transition(
        &session,
        ReasoningPrimitive::Inference,
        "project next temperature",
        vec![],
        vec![ev_inf.clone()],
        vec![],
        vec![],
        true,
        "",
        vec!["inference".to_string()],
    )?;

    // 4. VALIDATION
    let val_ok = condition != "ablation_fail_validation";
    session = runtime.transition(
        &session,
        ReasoningPrimitive::Validation,
        "consistency check",
        vec![],
        vec![],
        vec![],
        vec![],
        val_ok,
        if val_ok { "" } else { "projection diverges" },
        vec!["validation".to_string()],
    )?;

    // 5. COMMIT
    let cand = Candidate {
        candidate_id: runtime.id_factory.generate("cand")?,
        description: "accept cooling trajectory".to_string(),
        score: if val_ok { 0.82 } else { 0.3 },
        confidence: if val_ok { 0.78 } else { 0.35 },
        risk: 0.0,
        supporting_evidence_ids: vec![ev_obs.evidence_id, ev_inf.evidence_id],
        metadata: BTreeMap::new(),
    };

    session = runtime.transition(
        &session,
        ReasoningPrimitive::Commit,
        "emit candidate for DCK",
        vec![],
        vec![],
        vec![cand],
        vec![],
        true,
        "",
        vec!["commit".to_string(), "to_dck".to_string()],
    )?;

    Ok(session)
}

// =============================================================================
// Unit Tests for Absolute Determinism & Isolation
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absolute_determinism() {
        let session_a = run_condition(999, "test_cond").unwrap();
        let session_b = run_condition(999, "test_cond").unwrap();

        let json_a = serde_json::to_string(&session_a).unwrap();
        let json_b = serde_json::to_string(&session_b).unwrap();

        assert_eq!(
            json_a, json_b,
            "Serialized session JSON must match exactly byte-for-byte!"
        );
    }

    #[test]
    fn test_delta_action_removed_kind_isolation() {
        let runtime = LRPRuntime::new(123, None);
        let mut session = runtime
            .create_session("p1", vec![], vec![], None, "exp", "cond")
            .unwrap();

        let shared_id = "SHARED_001".to_string();

        // Add same-id Context and Evidence
        let ctx = ContextNode {
            node_id: shared_id.clone(),
            kind: "env".to_string(),
            payload: serde_json::json!("prod"),
            metadata: BTreeMap::new(),
            parent_ids: vec![],
        };
        let ev = Evidence::new(
            shared_id.clone(),
            EvidenceKind::Fact,
            serde_json::json!({}),
            "src".to_string(),
            1.0,
            vec![],
            runtime.clock.now().unwrap(),
        );

        session = runtime
            .transition(
                &session,
                ReasoningPrimitive::Observe,
                "add both",
                vec![ctx],
                vec![ev],
                vec![],
                vec![],
                true,
                "",
                vec![],
            )
            .unwrap();

        assert_eq!(session.current_state().context.nodes.len(), 1);
        assert_eq!(session.current_state().evidence.len(), 1);

        // Remove ONLY Evidence (kind-isolated)
        let remove_delta = PLPDelta {
            action: DeltaAction::Removed,
            kind: DeltaKind::EvidenceChange,
            payload: TypedPayload::Raw(serde_json::Value::Null),
            target_id: Some(shared_id.clone()),
            metadata: BTreeMap::new(),
        };

        session = runtime
            .transition(
                &session,
                ReasoningPrimitive::Rollback,
                "remove evidence only",
                vec![],
                vec![],
                vec![],
                vec![remove_delta],
                true,
                "",
                vec![],
            )
            .unwrap();

        // Evidence gone, Context remains
        assert_eq!(session.current_state().evidence.len(), 0);
        assert_eq!(session.current_state().context.nodes.len(), 1);
        assert_eq!(
            session.current_state().context.nodes[0].node_id,
            shared_id
        );
    }

    #[test]
    fn test_observer_panic_isolation() {
        let runtime = LRPRuntime::new(42, None).with_observers(vec![
            Arc::new(LatencyObserver),
            Arc::new(PanickingObserver),
            Arc::new(MetricObserver),
        ]);

        let mut session = runtime
            .create_session("panic_test", vec![], vec![], None, "exp", "cond")
            .unwrap();

        // This must NOT panic the whole process
        session = runtime
            .transition(
                &session,
                ReasoningPrimitive::Observe,
                "trigger observers",
                vec![],
                vec![],
                vec![],
                vec![],
                true,
                "",
                vec![],
            )
            .unwrap();

        // 3 observers → 3 records
        assert_eq!(session.observer_records.len(), 3);

        // Middle one is the panicking observer
        assert!(session.observer_records[0].error.is_none());
        assert!(session.observer_records[1].error.is_some());
        assert!(session.observer_records[1]
            .error
            .as_ref()
            .unwrap()
            .contains("panicked"));
        assert!(session.observer_records[2].error.is_none());
    }

    #[test]
    fn test_fork_and_replay_determinism() {
        let session = run_condition(777, "baseline").unwrap();

        // Replay to full should be identical
        let replayed = ReplayEngine
            .replay(&session, None)
            .unwrap();
        let json_orig = serde_json::to_string(&session).unwrap();
        let json_rep = serde_json::to_string(&replayed).unwrap();
        assert_eq!(json_orig, json_rep);

        // Fork at step 2
        let runtime = LRPRuntime::new(777, None);
        let forked = ReplayEngine
            .fork(&session, 2, "forked_cond", &runtime.id_factory)
            .unwrap();

        assert_eq!(forked.transitions.len(), 3); // 0..=2
        assert!(forked.condition.contains("forked") || forked.condition == "forked_cond");

        // Independent continuation of fork still deterministic
        let forked2 = run_condition(777, "baseline").unwrap();
        let forked_again = ReplayEngine
            .fork(&forked2, 2, "forked_cond", &runtime.id_factory)
            .unwrap();
        assert_eq!(
            serde_json::to_string(forked.current_state()).unwrap(),
            serde_json::to_string(forked_again.current_state()).unwrap()
        );
    }

    #[test]
    fn test_no_wall_clock_leak() {
        // Even if we force many operations, timestamps stay logical
        let runtime = LRPRuntime::new(1, Some(DeterministicClock::new(
            Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
            10,
        )));
        let session = runtime
            .create_session("t", vec![], vec![], None, "", "")
            .unwrap();
        // created_at must be the fixed start, not real now
        assert_eq!(
            session.created_at,
            Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
        );
    }
}

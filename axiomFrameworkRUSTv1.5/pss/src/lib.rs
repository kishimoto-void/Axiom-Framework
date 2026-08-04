use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// グローバルスレッドセーフ カウンタ（Unique ID 生成用）
static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);

// =============================================================================
// Custom Error
// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PssError {
    SerializationFailed(String),
    DeserializationFailed(String),
    ValidationError(String),
    InvalidConfiguration(String),
}

impl fmt::Display for PssError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PssError::SerializationFailed(msg) => write!(f, "[PSS Serialization Error] {}", msg),
            PssError::DeserializationFailed(msg) => write!(f, "[PSS Deserialization Error] {}", msg),
            PssError::ValidationError(msg) => write!(f, "[PSS Validation Error] {}", msg),
            PssError::InvalidConfiguration(msg) => write!(f, "[PSS Config Error] {}", msg),
        }
    }
}

impl std::error::Error for PssError {}

// =============================================================================
// Helper Functions
// =============================================================================

/// 文字数上限で安全に切り詰める（UTF-8 境界考慮）
pub fn safe_truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// バイト数上限で安全に切り詰める（UTF-8 境界を破壊しない）
pub fn safe_truncate_bytes(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut idx = max_bytes;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    &s[..idx]
}

// =============================================================================
// Strict Enums
// =============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[serde(rename = "1_clarify")]
    Clarify,
    #[serde(rename = "2_confirm")]
    Confirm,
    #[serde(rename = "3_answer")]
    Answer,
}

impl Phase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Phase::Clarify => "1_clarify",
            Phase::Confirm => "2_confirm",
            Phase::Answer => "3_answer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    #[default]
    Pass,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Critical,
    High,
    Normal,
    Low,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Critical => "critical",
            Priority::High => "high",
            Priority::Normal => "normal",
            Priority::Low => "low",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingStance {
    Analytical,
    Creative,
    Cautious,
    Speed,
    Balanced,
}

impl ThinkingStance {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThinkingStance::Analytical => "analytical",
            ThinkingStance::Creative => "creative",
            ThinkingStance::Cautious => "cautious",
            ThinkingStance::Speed => "speed",
            ThinkingStance::Balanced => "balanced",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Perspective {
    Researcher,
    Reviewer,
    Implementer,
    Educator,
    Advisor,
    Custom,
}

impl Perspective {
    pub fn as_str(&self) -> &'static str {
        match self {
            Perspective::Researcher => "researcher",
            Perspective::Reviewer => "reviewer",
            Perspective::Implementer => "implementer",
            Perspective::Educator => "educator",
            Perspective::Advisor => "advisor",
            Perspective::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingDepth {
    Brief,
    Normal,
    Deep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningBias {
    Balanced,
    ObservationFirst,
    HypothesisFirst,
    RiskFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceLevel {
    High,
    Medium,
    Low,
    None,
}

impl EvidenceLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            EvidenceLevel::High => "high",
            EvidenceLevel::Medium => "medium",
            EvidenceLevel::Low => "low",
            EvidenceLevel::None => "none",
        }
    }

    pub fn rank(&self) -> u8 {
        match self {
            EvidenceLevel::None => 0,
            EvidenceLevel::Low => 1,
            EvidenceLevel::Medium => 2,
            EvidenceLevel::High => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubMissionKind {
    GatherInfo,
    RiskScan,
    Alternatives,
    AskMissing,
    Custom,
}

impl SubMissionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubMissionKind::GatherInfo => "gather_info",
            SubMissionKind::RiskScan => "risk_scan",
            SubMissionKind::Alternatives => "alternatives",
            SubMissionKind::AskMissing => "ask_missing",
            SubMissionKind::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateCode {
    GateOk,
    GateMissionGoalEmpty,
    GateAskMissingPending,
    GateKnowledgeEmpty,
    GateScopeNotAgreed,
    GateScopeUndefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintKind {
    Hard,
    Soft,
    Assumption,
    Risk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriticismLevel {
    #[serde(rename = "0_off")]
    Off,
    #[serde(rename = "1_normal")]
    Normal,
    #[serde(rename = "2_strict")]
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BehaviorAction {
    Ask,
    Stop,
    StateUnknown,
    MarkAssumption,
    StateConfidence,
    Proceed,
}

impl BehaviorAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            BehaviorAction::Ask => "ask",
            BehaviorAction::Stop => "stop",
            BehaviorAction::StateUnknown => "state_unknown",
            BehaviorAction::MarkAssumption => "mark_assumption",
            BehaviorAction::StateConfidence => "state_confidence",
            BehaviorAction::Proceed => "proceed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Markdown,
    Json,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputLanguage {
    Ja,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdGenerationMode {
    /// 入力パラメータに基づく決定的ハッシュ ID (例: pss-det-1234567890abcdef)
    Deterministic,
    /// スレッドセーフなアトミックカウンタ＋時刻による衝突回避 ID (例: pss-uniq-...)
    Unique,
}

// =============================================================================
// Structs
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MainMission {
    pub goal: String,
    pub success_criteria: Vec<String>,
    pub priority: Priority,
}

impl Default for MainMission {
    fn default() -> Self {
        Self {
            goal: String::new(),
            success_criteria: Vec::new(),
            priority: Priority::Critical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubMission {
    pub kind: SubMissionKind,
    pub description: String,
    pub done: bool,
    pub priority: Priority,
}

impl SubMission {
    pub fn is_optional(&self) -> bool {
        matches!(self.priority, Priority::Low | Priority::Normal)
    }

    pub fn is_required(&self) -> bool {
        matches!(self.priority, Priority::Critical | Priority::High)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Mission {
    pub main: MainMission,
    pub subs: Vec<SubMission>,
}

impl Mission {
    /// 参照イテレータを返し、メモリアロケーションを回避
    pub fn required_subs(&self) -> impl Iterator<Item = &SubMission> {
        self.subs.iter().filter(|s| s.is_required())
    }

    pub fn optional_subs(&self) -> impl Iterator<Item = &SubMission> {
        self.subs.iter().filter(|s| s.is_optional())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeState {
    pub observation: Vec<String>,
    pub inference: Vec<String>,
    pub assumption: Vec<String>,
    pub unknown: Vec<String>,
    pub missing: Vec<String>,
}

impl KnowledgeState {
    pub fn is_empty(&self) -> bool {
        self.observation.is_empty()
            && self.inference.is_empty()
            && self.assumption.is_empty()
            && self.unknown.is_empty()
            && self.missing.is_empty()
    }

    pub fn evidence_count(&self) -> usize {
        self.observation.len()
    }

    pub fn estimated_evidence_level(&self) -> EvidenceLevel {
        let n = self.evidence_count();
        if n >= 5 {
            EvidenceLevel::High
        } else if n >= 2 {
            EvidenceLevel::Medium
        } else if n >= 1 {
            EvidenceLevel::Low
        } else {
            EvidenceLevel::None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstraintSpec {
    pub statement: String,
    pub kind: ConstraintKind,
    pub priority: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Constraints {
    pub hard: Vec<ConstraintSpec>,
    pub soft: Vec<ConstraintSpec>,
    pub assumptions: Vec<ConstraintSpec>,
    pub risks: Vec<ConstraintSpec>,
}

impl Constraints {
    /// 参照イテレータを返し、メモリアロケーションを回避
    pub fn all(&self) -> impl Iterator<Item = &ConstraintSpec> {
        self.hard
            .iter()
            .chain(self.soft.iter())
            .chain(self.assumptions.iter())
            .chain(self.risks.iter())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Scope {
    pub in_scope: Vec<String>,
    pub out_of_scope: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThinkingProfile {
    pub stance: ThinkingStance,
    pub perspective: Perspective,
    pub depth: ThinkingDepth,
    pub reasoning_bias: ReasoningBias,
    pub evidence_level: EvidenceLevel,
    pub perspective_note: String,
}

impl Default for ThinkingProfile {
    fn default() -> Self {
        Self {
            stance: ThinkingStance::Balanced,
            perspective: Perspective::Advisor,
            depth: ThinkingDepth::Normal,
            reasoning_bias: ReasoningBias::Balanced,
            evidence_level: EvidenceLevel::None,
            perspective_note: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BehaviorRules {
    pub if_unknown: BehaviorAction,
    pub if_assumption: BehaviorAction,
    pub if_scope_violation: BehaviorAction,
    pub if_missing_required: BehaviorAction,
    pub if_low_confidence: BehaviorAction,
}

impl Default for BehaviorRules {
    fn default() -> Self {
        Self {
            if_unknown: BehaviorAction::StateUnknown,
            if_assumption: BehaviorAction::MarkAssumption,
            if_scope_violation: BehaviorAction::Stop,
            if_missing_required: BehaviorAction::Ask,
            if_low_confidence: BehaviorAction::StateConfidence,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Behavior {
    pub role: String,
    pub role_description: String,
    pub criticism_level: CriticismLevel,
    pub rules: BehaviorRules,
}

impl Default for Behavior {
    fn default() -> Self {
        Self {
            role: "collaborator".to_string(),
            role_description: String::new(),
            criticism_level: CriticismLevel::Normal,
            rules: BehaviorRules::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PredictionPolicy {
    pub allow_prediction: bool,
    pub minimum_evidence: EvidenceLevel,
    pub when_uncertain: BehaviorAction,
    pub show_confidence: bool,
    pub explain_reason: bool,
    pub refuse_if_below_minimum: bool,
}

impl Default for PredictionPolicy {
    fn default() -> Self {
        Self {
            allow_prediction: true,
            minimum_evidence: EvidenceLevel::Medium,
            when_uncertain: BehaviorAction::StateUnknown,
            show_confidence: true,
            explain_reason: true,
            refuse_if_below_minimum: true,
        }
    }
}

impl PredictionPolicy {
    pub fn allows_with(&self, evidence_level: &EvidenceLevel) -> bool {
        if !self.allow_prediction {
            return false;
        }
        evidence_level.rank() >= self.minimum_evidence.rank()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PredictionQuality {
    pub confidence: String,
    pub uncertainty: String,
    pub evidence_count: usize,
    pub evidence_level: EvidenceLevel,
    pub reason: String,
    pub is_prediction_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationCriterion {
    pub name: String,
    pub weight: f64,
    pub notes: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EvaluationCriteria {
    pub criteria: Vec<EvaluationCriterion>,
}

impl EvaluationCriteria {
    pub fn normalized(&self) -> Vec<EvaluationCriterion> {
        if self.criteria.is_empty() {
            return Vec::new();
        }

        let total: f64 = self
            .criteria
            .iter()
            .map(|c| {
                if c.weight.is_finite() && c.weight > 0.0 {
                    c.weight
                } else {
                    0.0
                }
            })
            .sum();

        let count = self.criteria.len() as f64;
        let default_weight = 1.0 / count;

        self.criteria
            .iter()
            .map(|c| {
                let valid_w = if c.weight.is_finite() && c.weight > 0.0 {
                    c.weight
                } else {
                    0.0
                };
                let norm_w = if total > 0.0 {
                    valid_w / total
                } else {
                    default_weight
                };
                EvaluationCriterion {
                    name: c.name.clone(),
                    weight: (norm_w * 10000.0).round() / 10000.0,
                    notes: c.notes.clone(),
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateResult {
    pub phase: Phase,
    pub can_proceed: bool,
    pub codes: Vec<GateCode>,
    pub reasons: Vec<String>,
    pub blocking: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseState {
    pub phase: Phase,
    pub cycle: u32,
    pub scope: String,
    pub scope_agreed: bool,
    pub last_gate: Option<GateResult>,
}

impl Default for PhaseState {
    fn default() -> Self {
        Self {
            phase: Phase::Clarify,
            cycle: 1,
            scope: String::new(),
            scope_agreed: false,
            last_gate: None,
        }
    }
}

pub fn evaluate_gate(spec: &ProblemSpecification) -> GateResult {
    let phase = spec.phase_state.phase;
    let mut codes = Vec::new();
    let mut reasons = Vec::new();
    let mut blocking = Vec::new();

    match phase {
        Phase::Clarify => {
            if spec.mission.main.goal.trim().is_empty() {
                codes.push(GateCode::GateMissionGoalEmpty);
                blocking.push("Main Mission の goal が空".to_string());
            }

            if spec.knowledge.is_empty() {
                codes.push(GateCode::GateKnowledgeEmpty);
                blocking.push("Knowledge が完全に空".to_string());
            }

            let pending: Vec<&SubMission> = spec
                .mission
                .subs
                .iter()
                .filter(|s| s.kind == SubMissionKind::AskMissing && !s.done && s.is_required())
                .collect();

            if !pending.is_empty() {
                codes.push(GateCode::GateAskMissingPending);
                let pending_names: Vec<String> = pending
                    .iter()
                    .map(|s| {
                        if s.description.is_empty() {
                            s.kind.as_str().to_string()
                        } else {
                            s.description.clone()
                        }
                    })
                    .collect();
                blocking.push(format!(
                    "必須の不足情報質問が未完了: {:?}",
                    pending_names
                ));
            }

            if blocking.is_empty() {
                codes.push(GateCode::GateOk);
                reasons.push("Main Mission と最低限の Knowledge が揃っている".to_string());
            }

            GateResult {
                phase,
                can_proceed: blocking.is_empty(),
                codes,
                reasons,
                blocking,
            }
        }
        Phase::Confirm => {
            if !spec.phase_state.scope_agreed {
                codes.push(GateCode::GateScopeNotAgreed);
                blocking.push("scope_agreed が False".to_string());
            }
            if spec.phase_state.scope.trim().is_empty()
                && spec.scope.in_scope.is_empty()
                && spec.scope.out_of_scope.is_empty()
            {
                codes.push(GateCode::GateScopeUndefined);
                blocking.push("確認すべき scope が未定義".to_string());
            }
            if blocking.is_empty() {
                codes.push(GateCode::GateOk);
                reasons.push("scope_agreed=True で確認済み".to_string());
            }
            GateResult {
                phase,
                can_proceed: blocking.is_empty(),
                codes,
                reasons,
                blocking,
            }
        }
        Phase::Answer => {
            codes.push(GateCode::GateOk);
            reasons.push("Answer フェーズ（回答出力可能）".to_string());
            GateResult {
                phase,
                can_proceed: true,
                codes,
                reasons,
                blocking,
            }
        }
    }
}

// =============================================================================
// Aggregate
// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Identity {
    pub id: String,
    pub title: String,
    pub domain: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputSpec {
    pub format: OutputFormat,
    pub language: OutputLanguage,
    pub include_pros_cons: bool,
    pub include_confidence: bool,
    pub include_needed_info: bool,
}

impl Default for OutputSpec {
    fn default() -> Self {
        Self {
            format: OutputFormat::Markdown,
            language: OutputLanguage::Ja,
            include_pros_cons: false,
            include_confidence: true,
            include_needed_info: true,
        }
    }
}

/// 問題解決仕様（Problem Specification Standard）のメイン構造体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProblemSpecification {
    pub schema: String,
    pub version: String,
    /// 作成時の UNIX エポック秒（決定性と型安全性を保証）
    pub created_at: u64,
    pub identity: Identity,
    pub mission: Mission,
    pub thinking_profile: ThinkingProfile,
    pub prediction_policy: PredictionPolicy,
    pub evaluation: EvaluationCriteria,
    pub knowledge: KnowledgeState,
    pub constraints: Constraints,
    pub scope: Scope,
    pub behavior: Behavior,
    pub phase_state: PhaseState,
    pub output: OutputSpec,
}

impl ProblemSpecification {
    pub fn assess_prediction_quality(&self) -> PredictionQuality {
        let level = self.knowledge.estimated_evidence_level();
        let count = self.knowledge.evidence_count();
        let allowed = self.prediction_policy.allows_with(&level);
        let (conf, unc) = match level {
            EvidenceLevel::High => ("high", "low"),
            EvidenceLevel::Medium => ("medium", "medium"),
            EvidenceLevel::Low => ("low", "high"),
            EvidenceLevel::None => ("unknown", "high"),
        };
        let reason = if !self.prediction_policy.allow_prediction {
            "予測がポリシーで禁止されている".to_string()
        } else if !allowed {
            format!(
                "根拠レベル {} が minimum_evidence={} 未満",
                level.as_str(),
                self.prediction_policy.minimum_evidence.as_str()
            )
        } else {
            format!("observation={} 件に基づく予測が許容される", count)
        };
        PredictionQuality {
            confidence: conf.to_string(),
            uncertainty: unc.to_string(),
            evidence_count: count,
            evidence_level: level,
            reason,
            is_prediction_allowed: allowed,
        }
    }

    pub fn evaluate_gate(&self) -> GateResult {
        evaluate_gate(self)
    }

    pub fn with_gate_evaluated(mut self) -> (Self, GateResult) {
        let result = evaluate_gate(&self);
        self.phase_state.last_gate = Some(result.clone());
        (self, result)
    }

    pub fn to_json(&self) -> Result<String, PssError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PssError::SerializationFailed(e.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, PssError> {
        serde_json::from_str(json)
            .map_err(|e| PssError::DeserializationFailed(e.to_string()))
    }

    pub fn summary(&self) -> String {
        let pq = self.assess_prediction_quality();
        let mut lines = vec![
            format!("[PSS ProblemSpecification v{}]", self.version),
            format!("ID          : {}", self.identity.id),
            format!("Created At  : {} (UNIX)", self.created_at),
            format!("Title       : {}", self.identity.title),
            format!(
                "Main Mission: {} [{}]",
                self.mission.main.goal,
                self.mission.main.priority.as_str()
            ),
            format!(
                "SubMissions : req={} opt={}",
                self.mission.required_subs().count(),
                self.mission.optional_subs().count()
            ),
            format!("Phase       : {}", self.phase_state.phase.as_str()),
            format!(
                "Stance/Persp: {} / {}",
                self.thinking_profile.stance.as_str(),
                self.thinking_profile.perspective.as_str()
            ),
            format!(
                "Pred allow  : {} ({}, n={})",
                pq.is_prediction_allowed,
                pq.evidence_level.as_str(),
                pq.evidence_count
            ),
        ];

        if !self.evaluation.criteria.is_empty() {
            let crit: Vec<String> = self
                .evaluation
                .normalized()
                .iter()
                .map(|c| format!("{}={:.2}", c.name, c.weight))
                .collect();
            lines.push(format!("Eval        : {}", crit.join(", ")));
        }

        if let Some(ref gate) = self.phase_state.last_gate {
            lines.push(format!(
                "Gate        : proceed={} codes={:?}",
                gate.can_proceed, gate.codes
            ));
        }
        lines.join("\n")
    }
}

// =============================================================================
// Builder
// =============================================================================
pub struct ProblemBuilder {
    id_mode: IdGenerationMode,
    custom_id: Option<String>,
    custom_created_at: Option<u64>,
    identity: Identity,
    mission: Mission,
    thinking: ThinkingProfile,
    prediction: PredictionPolicy,
    evaluation: EvaluationCriteria,
    knowledge: KnowledgeState,
    constraints: Constraints,
    scope: Scope,
    behavior: Behavior,
    phase: PhaseState,
    output: OutputSpec,
}

impl Default for ProblemBuilder {
    fn default() -> Self {
        Self {
            id_mode: IdGenerationMode::Deterministic,
            custom_id: None,
            custom_created_at: None,
            identity: Identity::default(),
            mission: Mission::default(),
            thinking: ThinkingProfile::default(),
            prediction: PredictionPolicy::default(),
            evaluation: EvaluationCriteria::default(),
            knowledge: KnowledgeState::default(),
            constraints: Constraints::default(),
            scope: Scope::default(),
            behavior: Behavior::default(),
            phase: PhaseState::default(),
            output: OutputSpec::default(),
        }
    }
}

impl ProblemBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id_mode(mut self, mode: IdGenerationMode) -> Self {
        self.id_mode = mode;
        self
    }

    pub fn custom_id(mut self, id: &str) -> Self {
        self.custom_id = Some(id.to_string());
        self
    }

    pub fn custom_created_at(mut self, timestamp: u64) -> Self {
        self.custom_created_at = Some(timestamp);
        self
    }

    pub fn identity(mut self, title: &str, domain: &str, description: &str) -> Self {
        self.identity.title = title.to_string();
        self.identity.domain = domain.to_string();
        self.identity.description = description.to_string();
        self
    }

    pub fn main_mission(
        mut self,
        goal: &str,
        success_criteria: &[&str],
        priority: Priority,
    ) -> Self {
        self.mission.main = MainMission {
            goal: goal.to_string(),
            success_criteria: success_criteria.iter().map(|s| s.to_string()).collect(),
            priority,
        };
        self
    }

    pub fn add_sub_mission(
        mut self,
        kind: SubMissionKind,
        description: &str,
        priority: Priority,
        done: bool,
    ) -> Self {
        self.mission.subs.push(SubMission {
            kind,
            description: description.to_string(),
            priority,
            done,
        });
        self
    }

    pub fn add_constraint(mut self, statement: &str, kind: ConstraintKind, priority: i32) -> Self {
        let c = ConstraintSpec {
            statement: statement.to_string(),
            kind,
            priority,
        };
        match kind {
            ConstraintKind::Hard => self.constraints.hard.push(c),
            ConstraintKind::Soft => self.constraints.soft.push(c),
            ConstraintKind::Assumption => self.constraints.assumptions.push(c),
            ConstraintKind::Risk => self.constraints.risks.push(c),
        }
        self
    }

    pub fn add_default_safety_constraints(self) -> Self {
        self.add_constraint("推測しない。不明な点は不明と明示する。", ConstraintKind::Hard, 10)
            .add_constraint("不可能なことは不可能と言う。", ConstraintKind::Hard, 10)
            .add_constraint("制約違反を隠さない。", ConstraintKind::Hard, 10)
            .add_constraint("不足情報を明示する。", ConstraintKind::Hard, 9)
    }

    pub fn scope(mut self, in_scope: &[&str], out_of_scope: &[&str]) -> Self {
        self.scope.in_scope = in_scope.iter().map(|s| s.to_string()).collect();
        self.scope.out_of_scope = out_of_scope.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn knowledge(
        mut self,
        observation: &[&str],
        inference: &[&str],
        assumption: &[&str],
        unknown: &[&str],
        missing: &[&str],
    ) -> Self {
        self.knowledge.observation = observation.iter().map(|s| s.to_string()).collect();
        self.knowledge.inference = inference.iter().map(|s| s.to_string()).collect();
        self.knowledge.assumption = assumption.iter().map(|s| s.to_string()).collect();
        self.knowledge.unknown = unknown.iter().map(|s| s.to_string()).collect();
        self.knowledge.missing = missing.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn thinking_profile(
        mut self,
        stance: ThinkingStance,
        perspective: Perspective,
        depth: ThinkingDepth,
        reasoning_bias: ReasoningBias,
        evidence_level: EvidenceLevel,
        perspective_note: &str,
    ) -> Self {
        self.thinking = ThinkingProfile {
            stance,
            perspective,
            depth,
            reasoning_bias,
            evidence_level,
            perspective_note: perspective_note.to_string(),
        };
        self
    }

    pub fn prediction_policy(
        mut self,
        allow_prediction: bool,
        minimum_evidence: EvidenceLevel,
        when_uncertain: BehaviorAction,
        show_confidence: bool,
        explain_reason: bool,
        refuse_if_below_minimum: bool,
    ) -> Self {
        self.prediction = PredictionPolicy {
            allow_prediction,
            minimum_evidence,
            when_uncertain,
            show_confidence,
            explain_reason,
            refuse_if_below_minimum,
        };
        self
    }

    pub fn evaluation(mut self, criteria: Vec<(&str, f64, &str)>) -> Self {
        self.evaluation = EvaluationCriteria {
            criteria: criteria
                .into_iter()
                .map(|(name, weight, notes)| EvaluationCriterion {
                    name: name.to_string(),
                    weight,
                    notes: notes.to_string(),
                })
                .collect(),
        };
        self
    }

    pub fn phase(
        mut self,
        phase: Phase,
        cycle: u32,
        scope: &str,
        scope_agreed: bool,
    ) -> Self {
        self.phase = PhaseState {
            phase,
            cycle,
            scope: scope.to_string(),
            scope_agreed,
            last_gate: None,
        };
        self
    }

    /// ビルド時に単一のタイムスタンプと ID を決定的に一括生成する
    pub fn build(mut self) -> ProblemSpecification {
        let mut unknown_set: HashSet<String> = self.knowledge.unknown.iter().cloned().collect();
        for m in &self.knowledge.missing {
            if !unknown_set.contains(m) {
                self.knowledge.unknown.push(m.clone());
                unknown_set.insert(m.clone());
            }
        }

        // 1箇所で時刻を決定
        let created_at = self.custom_created_at.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

        // ID の決定または一意生成
        let id = match self.custom_id {
            Some(id) => id,
            None => match self.id_mode {
                IdGenerationMode::Deterministic => {
                    let mut hasher = DefaultHasher::new();
                    self.identity.title.hash(&mut hasher);
                    self.identity.domain.hash(&mut hasher);
                    self.mission.main.goal.hash(&mut hasher);
                    created_at.hash(&mut hasher);
                    format!("pss-det-{:016x}", hasher.finish())
                }
                IdGenerationMode::Unique => {
                    let count = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
                    let nanos = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .subsec_nanos();
                    format!("pss-uniq-{:012x}-{:08x}-{:04x}", created_at, nanos, count % 0xFFFF)
                }
            },
        };

        self.identity.id = id;
        self.identity.version = "1.0.0-rc1".to_string();

        ProblemSpecification {
            schema: "pss.problem_specification/1.0".to_string(),
            version: "1.0.0-rc1".to_string(),
            created_at,
            identity: self.identity,
            mission: self.mission,
            thinking_profile: self.thinking,
            prediction_policy: self.prediction,
            evaluation: self.evaluation,
            knowledge: self.knowledge,
            constraints: self.constraints,
            scope: self.scope,
            behavior: self.behavior,
            phase_state: self.phase,
            output: self.output,
        }
    }
}

// =============================================================================
// Validator（運用層）
// =============================================================================
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ValidationReport {
    pub overall: Severity,
    pub findings: Vec<Finding>,
}

impl ValidationReport {
    pub fn add(&mut self, f: Finding) {
        if f.severity == Severity::Error {
            self.overall = Severity::Error;
        } else if f.severity == Severity::Warn && self.overall != Severity::Error {
            self.overall = Severity::Warn;
        }
        self.findings.push(f);
    }

    pub fn summary(&self) -> String {
        let mut lines = vec![format!("[Validation] Overall: {:?}", self.overall)];
        for f in &self.findings {
            lines.push(format!("  [{:?}] {}: {}", f.severity, f.code, f.message));
            if !f.suggestion.is_empty() {
                lines.push(format!("    → {}", f.suggestion));
            }
        }
        if self.findings.is_empty() {
            lines.push("  No issues.".to_string());
        }
        lines.join("\n")
    }
}

pub fn validate(spec: &ProblemSpecification) -> ValidationReport {
    let mut report = ValidationReport {
        overall: Severity::Pass,
        findings: Vec::new(),
    };

    if spec.schema != "pss.problem_specification/1.0" {
        report.add(Finding {
            code: "INVALID_SCHEMA".to_string(),
            severity: Severity::Error,
            message: format!("未知のスキーマ: {}", spec.schema),
            suggestion: "schema を pss.problem_specification/1.0 に設定してください".to_string(),
        });
    }

    if !spec.identity.id.starts_with("pss-") {
        report.add(Finding {
            code: "INVALID_ID_FORMAT".to_string(),
            severity: Severity::Error,
            message: format!("IDフォーマット不正: {}", spec.identity.id),
            suggestion: "IDは 'pss-' から始まる必要があります".to_string(),
        });
    }

    if spec.identity.title.trim().is_empty() {
        report.add(Finding {
            code: "IDENTITY_TITLE_MISSING".to_string(),
            severity: Severity::Error,
            message: "title が空".to_string(),
            suggestion: "タイトルを設定してください".to_string(),
        });
    }

    if spec.mission.main.goal.trim().is_empty() {
        report.add(Finding {
            code: "MISSION_GOAL_MISSING".to_string(),
            severity: Severity::Error,
            message: "Main Mission goal が空".to_string(),
            suggestion: "達成目標を記載してください".to_string(),
        });
    }

    if !spec.evaluation.criteria.is_empty() {
        let total_weight: f64 = spec
            .evaluation
            .criteria
            .iter()
            .map(|c| if c.weight.is_finite() && c.weight > 0.0 { c.weight } else { 0.0 })
            .sum();

        if total_weight <= 0.0 {
            report.add(Finding {
                code: "EVALUATION_WEIGHTS_INVALID".to_string(),
                severity: Severity::Error,
                message: "EvaluationCriteria の重み合計が 0 以下または無効値".to_string(),
                suggestion: "正の数値の重みを設定してください".to_string(),
            });
        }
    }

    report
}

// =============================================================================
// Compiler & Adapter
// =============================================================================
pub fn compile_for_generic(spec: &ProblemSpecification, mode: &str) -> String {
    let pq = spec.assess_prediction_quality();
    let gate = &spec.phase_state.last_gate;
    let mode_clean = mode.to_lowercase();
    let is_strict = mode_clean == "strict";

    let intro = "あなたは優秀な問題解決のパートナーです。\n以下は、今回の問題をできるだけ正確・安全に扱うための短い仕様です。\n推測で穴埋めせず、仕様に書かれた条件の範囲で答えてください。\n";

    let mut core_lines = vec![format!(
        "【やること】{}",
        if spec.mission.main.goal.is_empty() {
            "(未設定)"
        } else {
            &spec.mission.main.goal
        }
    )];

    if !spec.mission.main.success_criteria.is_empty() {
        core_lines.push(format!(
            "【成功の目安】{}",
            spec.mission.main.success_criteria.join(" / ")
        ));
    }

    if !spec.scope.in_scope.is_empty() || !spec.scope.out_of_scope.is_empty() {
        if !spec.scope.in_scope.is_empty() {
            core_lines.push(format!("【範囲内】{}", spec.scope.in_scope.join("、")));
        }
        if !spec.scope.out_of_scope.is_empty() {
            core_lines.push(format!("【範囲外・触れない】{}", spec.scope.out_of_scope.join("、")));
        }
    }

    if !spec.knowledge.observation.is_empty() {
        core_lines.push("【分かっていること】".to_string());
        for x in &spec.knowledge.observation {
            core_lines.push(format!("  - {}", x));
        }
    }

    let mut unknowns = spec.knowledge.unknown.clone();
    for m in &spec.knowledge.missing {
        if !unknowns.contains(m) {
            unknowns.push(m.clone());
        }
    }

    if !unknowns.is_empty() {
        core_lines.push("【まだ分からないこと】".to_string());
        for x in &unknowns {
            core_lines.push(format!("  - {}", x));
        }
    }

    let pred = if pq.is_prediction_allowed {
        format!(
            "根拠は足りているので、判断してよい（確信度={}）。理由も短く添える。",
            pq.confidence
        )
    } else {
        match spec.prediction_policy.when_uncertain {
            BehaviorAction::Ask => "根拠が足りないので、断定せず必要な情報を質問する。".to_string(),
            BehaviorAction::StateUnknown => "根拠が足りないので、「現時点では判断できない」と明示する。".to_string(),
            action => format!("根拠が足りないので、方針「{}」に従う（断定しない）。", action.as_str()),
        }
    };

    core_lines.push(format!("【予測・断定】{}", pred));

    if let Some(ref g) = gate {
        if !g.can_proceed {
            core_lines.push("【進行】まだ先に進まない。足りない点を確認・質問する。".to_string());
            if !g.blocking.is_empty() {
                core_lines.push(format!("  止めている理由: {}", g.blocking.join(" / ")));
            }
        } else {
            match spec.phase_state.phase {
                Phase::Clarify => {
                    core_lines.push("【進行】整理・確認フェーズ。不明点があれば先に聞く。".to_string())
                }
                Phase::Confirm => core_lines.push("【進行】確認済みの範囲で答えてよい。".to_string()),
                Phase::Answer => core_lines.push("【進行】回答フェーズ。仕様の範囲で結論を出してよい。".to_string()),
            }
        }
    }

    let body = core_lines.join("\n");

    if !is_strict {
        return format!("{}\n{}\n", intro, body);
    }

    format!("{}\n{}\n\n[STRICT_MODE_ACTIVE]\n", intro, body)
}

// =============================================================================
// Unit Tests (Comprehensive Test Suite)
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_roundtrip() {
        let spec = ProblemBuilder::new()
            .identity("ラウンドトリップテスト", "test", "説明")
            .main_mission("目標達成", &["基準1"], Priority::Critical)
            .knowledge(&["観測1"], &["推論1"], &["仮定1"], &["不明1"], &[])
            .build();

        let json = spec.to_json().expect("Serialization failed");
        let deserialized = ProblemSpecification::from_json(&json).expect("Deserialization failed");

        assert_eq!(spec, deserialized);
    }

    #[test]
    fn test_determinism() {
        let spec1 = ProblemBuilder::new()
            .id_mode(IdGenerationMode::Deterministic)
            .custom_created_at(1700000000)
            .identity("決定性テスト", "domain", "desc")
            .main_mission("目標", &[], Priority::Critical)
            .build();

        let spec2 = ProblemBuilder::new()
            .id_mode(IdGenerationMode::Deterministic)
            .custom_created_at(1700000000)
            .identity("決定性テスト", "domain", "desc")
            .main_mission("目標", &[], Priority::Critical)
            .build();

        assert_eq!(spec1.identity.id, spec2.identity.id);
        assert_eq!(spec1.to_json().unwrap(), spec2.to_json().unwrap());
    }

    #[test]
    fn test_unique_id_no_collision() {
        let mut ids = HashSet::new();
        for _ in 0..1000 {
            let spec = ProblemBuilder::new()
                .id_mode(IdGenerationMode::Unique)
                .identity("一意性テスト", "test", "")
                .build();
            assert!(ids.insert(spec.identity.id));
        }
    }

    #[test]
    fn test_gate_all_phases() {
        // Clarify Pass / Fail
        let (_spec_clarify_fail, gate) = ProblemBuilder::new()
            .phase(Phase::Clarify, 1, "", false)
            .build()
            .with_gate_evaluated();
        assert!(!gate.can_proceed);

        let (_spec_clarify_pass, gate) = ProblemBuilder::new()
            .main_mission("目標あり", &[], Priority::Critical)
            .knowledge(&["観測あり"], &[], &[], &[], &[])
            .phase(Phase::Clarify, 1, "", false)
            .build()
            .with_gate_evaluated();
        assert!(gate.can_proceed);

        // Confirm Pass / Fail
        let (_, gate) = ProblemBuilder::new()
            .phase(Phase::Confirm, 1, "スコープ定義済", false)
            .build()
            .with_gate_evaluated();
        assert!(!gate.can_proceed); // scope_agreed = false

        let (_, gate) = ProblemBuilder::new()
            .phase(Phase::Confirm, 1, "スコープ定義済", true)
            .build()
            .with_gate_evaluated();
        assert!(gate.can_proceed);

        // Answer Pass
        let (_, gate) = ProblemBuilder::new()
            .phase(Phase::Answer, 1, "", false)
            .build()
            .with_gate_evaluated();
        assert!(gate.can_proceed);
    }

    #[test]
    fn test_safe_truncate_chars_and_bytes() {
        let ja_text = "こんにちは、世界！Rust言語";
        assert_eq!(safe_truncate_chars(ja_text, 5), "こんにちは");

        let truncated_bytes = safe_truncate_bytes(ja_text, 10);
        assert!(std::str::from_utf8(truncated_bytes.as_bytes()).is_ok());
    }

    #[test]
    fn test_evaluation_criteria_edge_cases() {
        let criteria = EvaluationCriteria {
            criteria: vec![
                EvaluationCriterion {
                    name: "正数".to_string(),
                    weight: 2.0,
                    notes: "".to_string(),
                },
                EvaluationCriterion {
                    name: "NaN".to_string(),
                    weight: f64::NAN,
                    notes: "".to_string(),
                },
                EvaluationCriterion {
                    name: "Inf".to_string(),
                    weight: f64::INFINITY,
                    notes: "".to_string(),
                },
                EvaluationCriterion {
                    name: "負数".to_string(),
                    weight: -5.0,
                    notes: "".to_string(),
                },
            ],
        };

        let norm = criteria.normalized();
        assert_eq!(norm.len(), 4);
        assert_eq!(norm[0].weight, 1.0); // 唯一有効な重みなので 100%
        assert_eq!(norm[1].weight, 0.0);
        assert_eq!(norm[2].weight, 0.0);
        assert_eq!(norm[3].weight, 0.0);
    }

    #[test]
    fn test_builder_overrides() {
        let custom_id = "pss-custom-12345";
        let custom_time = 1600000000;

        let spec = ProblemBuilder::new()
            .custom_id(custom_id)
            .custom_created_at(custom_time)
            .identity("タイトル", "ドメイン", "説明")
            .build();

        assert_eq!(spec.identity.id, custom_id);
        assert_eq!(spec.created_at, custom_time);
    }

    #[test]
    fn test_iterator_zero_alloc() {
        let spec = ProblemBuilder::new()
            .add_sub_mission(SubMissionKind::AskMissing, "必須1", Priority::Critical, false)
            .add_sub_mission(SubMissionKind::Alternatives, "任意1", Priority::Low, false)
            .build();

        let req_count = spec.mission.required_subs().count();
        let opt_count = spec.mission.optional_subs().count();

        assert_eq!(req_count, 1);
        assert_eq!(opt_count, 1);
    }

    #[test]
    fn test_validation_rules() {
        let mut spec = ProblemBuilder::new().build();
        spec.schema = "invalid_schema".to_string();
        spec.identity.id = "invalid_id_prefix".to_string();

        let report = validate(&spec);
        assert_eq!(report.overall, Severity::Error);
        assert!(report.findings.iter().any(|f| f.code == "INVALID_SCHEMA"));
        assert!(report.findings.iter().any(|f| f.code == "INVALID_ID_FORMAT"));
    }
}

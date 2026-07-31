#!/usr/bin/env python3
"""LRP (LLM Reasoning Protocol) — Single-file edition v1.2 (Research)

Reasoning Transition Protocol.
Intelligence Neutral / Model Neutral / Language Neutral.

> 実験は忠実に実際行って

v1.2 研究者向け強化点
--------------------
1. 形式的な遷移意味論の明示（primitive の契約・合成規則）
2. 決定論的 ID / 時計（multi-seed・再現実験用）
3. Evidence の provenance 連鎖
4. Transition / Session の定量メトリクス
5. 実験条件タグ（ablation / condition / seed）
6. History = Truth を崩さない Replay / Fork / Slice
7. 論文向けサマリー出力（primitive 分布・検証成功率・証拠成長）

設計原則（維持）
----------------
- LRP は知能を持たない
- LRP は推論を実行しない
- LRP は推論状態遷移だけを表現・記録・交換・再生する
- History = Transition 列が唯一の Truth
- Current は Cache
- Decision は DCK の責務（LRP は Candidate まで）
- PLP へ渡すのは typed Delta のみ

責務境界
--------
PSS  = Problem Definition   (What)
LRP  = Reasoning Transition (How the reasoning state changes)
PLP  = State Transport      (Transition as data)
DCK  = Convergence          (Accept / Reject / Decision)

Dependencies:
    pip install pydantic

Usage:
    python lrp_single.py
"""
from __future__ import annotations

import hashlib
import json
import math
import uuid
from collections import Counter
from datetime import datetime, timezone, timedelta
from enum import Enum
from typing import Any, Dict, List, Optional, Protocol, Sequence, Tuple
from pydantic import BaseModel, ConfigDict, Field, PrivateAttr, field_validator

__version__ = "1.2.0-research"


# =============================================================================
# Philosophy
# =============================================================================

PHILOSOPHY = (
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
)


# =============================================================================
# 1. Reasoning Primitives + formal contracts (research semantics)
# =============================================================================

class ReasoningPrimitive(str, Enum):
    """
    汎用推論プリミティブ。

    形式的契約（研究者向け）:
      OBSERVE    : 外部観測を Evidence として導入する（副作用なしを推奨）
      HYPOTHESIS : 未検証の主張を導入する（Assumption 相当）
      INFERENCE  : 既存 Evidence から導出する（派生）
      VALIDATION : 主張・導出の整合を検査する（真偽を記録、状態を壊さない）
      COMMIT     : Candidate を確定し DCK へ渡す境界
      FORK       : 履歴分岐（実験条件の分離）
      MERGE      : 分岐の統合（衝突解決は呼び出し側）
      ROLLBACK   : 指定 Transition 以前へ戻す（履歴は残す / または fork）
    """
    OBSERVE = "Observe"
    HYPOTHESIS = "Hypothesis"
    INFERENCE = "Inference"
    VALIDATION = "Validation"
    COMMIT = "Commit"
    FORK = "Fork"
    MERGE = "Merge"
    ROLLBACK = "Rollback"


# primitive → 期待される主効果（ドキュメント用。実行時強制はしない）
PRIMITIVE_CONTRACT: Dict[str, str] = {
    "Observe": "introduces external evidence",
    "Hypothesis": "introduces unverified claim",
    "Inference": "derives from existing evidence",
    "Validation": "records consistency check; does not invent facts",
    "Commit": "emits candidates for DCK; terminal-ish boundary",
    "Fork": "branches history for controlled comparison",
    "Merge": "joins branches; conflict policy external",
    "Rollback": "returns view to earlier transition index",
}


# =============================================================================
# 2. Determinism helpers (multi-seed / reproducibility)
# =============================================================================

class DeterministicClock:
    """固定開始時刻 + 固定ステップで再現可能な時刻列を生成する。"""

    def __init__(self, start: Optional[datetime] = None, step_seconds: float = 1.0):
        self._t = start or datetime(2026, 7, 31, 12, 0, 0, tzinfo=timezone.utc)
        self._step = timedelta(seconds=step_seconds)

    def __call__(self) -> datetime:
        now = self._t
        self._t = self._t + self._step
        return now


class DeterministicIDFactory:
    """seed 付きで ID を決定論的に生成する（実験再現用）。"""

    def __init__(self, seed: int = 0, prefix: str = "lrp"):
        self.seed = seed
        self.prefix = prefix
        self._counter = 0

    def __call__(self, kind: str) -> str:
        self._counter += 1
        raw = f"{self.prefix}:{self.seed}:{kind}:{self._counter}"
        h = hashlib.sha256(raw.encode("utf-8")).hexdigest()[:12]
        return f"{kind}_{self.seed}_{h}"


# =============================================================================
# 3. Evidence — payload Any + provenance chain
# =============================================================================

class EvidenceKind(str, Enum):
    FACT = "Fact"
    INFERENCE = "Inference"
    ASSUMPTION = "Assumption"
    RETRIEVED = "Retrieved"
    CALCULATED = "Calculated"
    OBSERVED = "Observed"
    TOOL_RESULT = "ToolResult"
    MEMORY = "Memory"
    GRAPH = "Graph"
    TENSOR = "Tensor"
    IMAGE = "Image"
    OTHER = "Other"


class Evidence(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid", arbitrary_types_allowed=True)

    evidence_id: str
    kind: EvidenceKind
    payload: Any
    source: str = ""
    confidence: float = Field(default=1.0, ge=0.0, le=1.0)
    # provenance: この証拠が依存する先行 evidence_id 列
    derived_from: Tuple[str, ...] = Field(default_factory=tuple)
    metadata: Dict[str, Any] = Field(default_factory=dict)
    content_hash: str = ""
    timestamp: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))

    @field_validator("content_hash", mode="before")
    @classmethod
    def _ensure_hash(cls, v: Any, info) -> str:
        if v:
            return str(v)
        try:
            raw = json.dumps(info.data.get("payload"), sort_keys=True, default=str)
        except Exception:
            raw = str(info.data.get("payload"))
        return hashlib.sha256(raw.encode("utf-8")).hexdigest()[:16]


# =============================================================================
# 4. Context graph
# =============================================================================

class ContextNode(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")

    node_id: str
    kind: str
    payload: Any = None
    metadata: Dict[str, Any] = Field(default_factory=dict)
    parent_ids: Tuple[str, ...] = Field(default_factory=tuple)


class ContextGraph(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")

    nodes: Tuple[ContextNode, ...] = Field(default_factory=tuple)

    def add(self, node: ContextNode) -> "ContextGraph":
        return self.model_copy(update={"nodes": self.nodes + (node,)})

    def remove(self, node_id: str) -> "ContextGraph":
        return self.model_copy(update={"nodes": tuple(n for n in self.nodes if n.node_id != node_id)})

    def merge(self, other: "ContextGraph") -> "ContextGraph":
        existing = {n.node_id for n in self.nodes}
        extra = tuple(n for n in other.nodes if n.node_id not in existing)
        return self.model_copy(update={"nodes": self.nodes + extra})

    def get(self, node_id: str) -> Optional[ContextNode]:
        for n in self.nodes:
            if n.node_id == node_id:
                return n
        return None

    def by_kind(self, kind: str) -> List[ContextNode]:
        return [n for n in self.nodes if n.kind == kind]


# =============================================================================
# 5. Capability / Contract / Candidate
# =============================================================================

class Capability(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    capability_id: str
    version: str = "1.0"
    constraints: Dict[str, Any] = Field(default_factory=dict)
    metadata: Dict[str, Any] = Field(default_factory=dict)


class Contract(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    protocol_id: str
    description: str = ""
    side_effect_free: bool = True
    required_capabilities: Tuple[str, ...] = Field(default_factory=tuple)
    input_schema_hint: str = ""
    output_schema_hint: str = ""


class Candidate(BaseModel):
    """LRP は Candidate まで。Decision は DCK。"""
    model_config = ConfigDict(frozen=True, extra="forbid")
    candidate_id: str
    description: str
    score: float = 0.0
    confidence: float = Field(default=0.5, ge=0.0, le=1.0)
    risk: float = Field(default=0.0, ge=0.0)
    supporting_evidence_ids: Tuple[str, ...] = Field(default_factory=tuple)
    metadata: Dict[str, Any] = Field(default_factory=dict)


# =============================================================================
# 6. Typed Delta
# =============================================================================

class DeltaKind(str, Enum):
    PARTICLE = "ParticleChange"
    EDGE = "EdgeChange"
    CAPABILITY = "CapabilityChange"
    MEMORY = "MemoryChange"
    CONTEXT = "ContextChange"
    EVIDENCE = "EvidenceChange"
    CANDIDATE = "CandidateChange"
    PRIMITIVE = "PrimitiveChange"
    METRIC = "MetricChange"
    CUSTOM = "Custom"


class PLPDelta(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    kind: DeltaKind
    payload: Any
    target_id: Optional[str] = None
    metadata: Dict[str, Any] = Field(default_factory=dict)


# =============================================================================
# 7. Transition — Truth + research fields
# =============================================================================

class ReasoningTransition(BaseModel):
    """
    原子単位。History の要素。
    parent_transition_id で分岐・派生を追跡可能。
    """
    model_config = ConfigDict(frozen=True, extra="forbid")

    transition_id: str
    primitive: ReasoningPrimitive
    before_state_id: str
    after_state_id: str
    operation: str
    deltas: Tuple[PLPDelta, ...] = Field(default_factory=tuple)
    produced_evidence_ids: Tuple[str, ...] = Field(default_factory=tuple)
    produced_candidate_ids: Tuple[str, ...] = Field(default_factory=tuple)
    contract_protocol_id: Optional[str] = None
    parent_transition_id: Optional[str] = None  # fork / derived
    validation_passed: bool = True
    validation_message: str = ""
    # experiment labels
    experiment_id: str = ""
    condition: str = ""          # e.g. "ablation_no_critic", "seed=3"
    tags: Tuple[str, ...] = Field(default_factory=tuple)
    timestamp: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    metadata: Dict[str, Any] = Field(default_factory=dict)


# =============================================================================
# 8. State (Cache) / Session
# =============================================================================

class ReasoningState(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")

    state_id: str
    context: ContextGraph = Field(default_factory=ContextGraph)
    evidence: Tuple[Evidence, ...] = Field(default_factory=tuple)
    candidates: Tuple[Candidate, ...] = Field(default_factory=tuple)
    last_primitive: Optional[ReasoningPrimitive] = None
    step_count: int = 0
    notes: str = ""
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ObserverRecord(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    record_id: str
    transition_id: str
    protocol_id: str
    metrics: Dict[str, float] = Field(default_factory=dict)
    timing_ms: float = 0.0
    resource_hint: Dict[str, Any] = Field(default_factory=dict)
    reason: str = ""
    timestamp: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ReasoningSession(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")

    session_id: str
    problem_id: str
    initial_state: ReasoningState
    transitions: Tuple[ReasoningTransition, ...] = Field(default_factory=tuple)
    observer_records: Tuple[ObserverRecord, ...] = Field(default_factory=tuple)
    capabilities: Tuple[Capability, ...] = Field(default_factory=tuple)
    contracts: Tuple[Contract, ...] = Field(default_factory=tuple)
    # experiment context
    experiment_id: str = ""
    seed: int = 0
    condition: str = ""
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    version: str = __version__

    _current_cache: Optional[ReasoningState] = PrivateAttr(default=None)

    @property
    def current_state(self) -> ReasoningState:
        if self._current_cache is not None:
            return self._current_cache
        return self.initial_state

    def with_cache(self, state: Optional[ReasoningState]) -> "ReasoningSession":
        obj = self.model_copy(deep=False)
        object.__setattr__(obj, "_current_cache", state)
        return obj


# =============================================================================
# 9. Quantitative metrics (research)
# =============================================================================

class SessionMetrics(BaseModel):
    """論文・実験ログ向けの定量指標。"""
    model_config = ConfigDict(frozen=True, extra="forbid")

    n_transitions: int = 0
    n_evidence: int = 0
    n_candidates: int = 0
    n_validations: int = 0
    n_validation_pass: int = 0
    validation_pass_rate: float = 0.0
    primitive_counts: Dict[str, int] = Field(default_factory=dict)
    evidence_growth: List[int] = Field(default_factory=list)  # 各 transition 後の累積
    mean_evidence_confidence: float = 0.0
    mean_candidate_confidence: float = 0.0
    n_observer_records: int = 0
    condition: str = ""
    seed: int = 0
    experiment_id: str = ""


def compute_session_metrics(session: ReasoningSession) -> SessionMetrics:
    prim_counts: Counter = Counter()
    n_val = 0
    n_val_pass = 0
    growth: List[int] = []
    ev_count = 0

    # approximate growth by produced_evidence_ids length accumulation
    for tr in session.transitions:
        prim_counts[tr.primitive.value] += 1
        if tr.primitive == ReasoningPrimitive.VALIDATION:
            n_val += 1
            if tr.validation_passed:
                n_val_pass += 1
        ev_count += len(tr.produced_evidence_ids)
        growth.append(ev_count)

    evidence = session.current_state.evidence
    candidates = session.current_state.candidates
    mean_ev_conf = (
        sum(e.confidence for e in evidence) / len(evidence) if evidence else 0.0
    )
    mean_cand_conf = (
        sum(c.confidence for c in candidates) / len(candidates) if candidates else 0.0
    )
    pass_rate = (n_val_pass / n_val) if n_val else 0.0

    return SessionMetrics(
        n_transitions=len(session.transitions),
        n_evidence=len(evidence),
        n_candidates=len(candidates),
        n_validations=n_val,
        n_validation_pass=n_val_pass,
        validation_pass_rate=pass_rate,
        primitive_counts=dict(prim_counts),
        evidence_growth=growth,
        mean_evidence_confidence=mean_ev_conf,
        mean_candidate_confidence=mean_cand_conf,
        n_observer_records=len(session.observer_records),
        condition=session.condition,
        seed=session.seed,
        experiment_id=session.experiment_id,
    )


def paper_summary(session: ReasoningSession) -> str:
    """実験ノート / 論文用の短い定量サマリー。"""
    m = compute_session_metrics(session)
    lines = [
        f"[LRP Session Metrics] experiment={m.experiment_id or '-'} condition={m.condition or '-'} seed={m.seed}",
        f"  transitions      : {m.n_transitions}",
        f"  evidence         : {m.n_evidence}  (mean conf={m.mean_evidence_confidence:.3f})",
        f"  candidates       : {m.n_candidates}  (mean conf={m.mean_candidate_confidence:.3f})",
        f"  validations      : {m.n_validations}  pass_rate={m.validation_pass_rate:.3f}",
        f"  primitive_counts : {m.primitive_counts}",
        f"  evidence_growth  : {m.evidence_growth}",
        f"  observer_records : {m.n_observer_records}",
    ]
    return "\n".join(lines)


# =============================================================================
# 10. Observers
# =============================================================================

class IObserver(Protocol):
    @property
    def protocol_id(self) -> str: ...
    async def observe(self, transition: ReasoningTransition, state: ReasoningState) -> ObserverRecord: ...


class LatencyObserver:
    protocol_id = "observer.latency.v1"

    async def observe(self, transition: ReasoningTransition, state: ReasoningState) -> ObserverRecord:
        return ObserverRecord(
            record_id=f"obs_{uuid.uuid4().hex[:10]}",
            transition_id=transition.transition_id,
            protocol_id=self.protocol_id,
            metrics={"step_count": float(state.step_count)},
            timing_ms=0.0,
            reason="latency stub",
        )


class MetricObserver:
    """各 transition で簡単な定量を残す研究用 Observer。"""
    protocol_id = "observer.metric.v1"

    async def observe(self, transition: ReasoningTransition, state: ReasoningState) -> ObserverRecord:
        return ObserverRecord(
            record_id=f"obs_{uuid.uuid4().hex[:10]}",
            transition_id=transition.transition_id,
            protocol_id=self.protocol_id,
            metrics={
                "n_evidence": float(len(state.evidence)),
                "n_candidates": float(len(state.candidates)),
                "validation_passed": 1.0 if transition.validation_passed else 0.0,
            },
            reason=f"{transition.primitive.value}:{transition.operation}",
        )


# =============================================================================
# 11. Managers
# =============================================================================

class TransitionManager:
    def apply(
        self,
        session: ReasoningSession,
        primitive: ReasoningPrimitive,
        operation: str,
        after_state: ReasoningState,
        deltas: Sequence[PLPDelta] = (),
        evidence_ids: Sequence[str] = (),
        candidate_ids: Sequence[str] = (),
        contract_protocol_id: Optional[str] = None,
        parent_transition_id: Optional[str] = None,
        validation_passed: bool = True,
        validation_message: str = "",
        tags: Sequence[str] = (),
        metadata: Optional[Dict[str, Any]] = None,
        id_factory: Optional[DeterministicIDFactory] = None,
        clock: Optional[DeterministicClock] = None,
    ) -> ReasoningSession:
        tid = id_factory("tr") if id_factory else f"tr_{uuid.uuid4().hex[:12]}"
        ts = clock() if clock else datetime.now(timezone.utc)
        transition = ReasoningTransition(
            transition_id=tid,
            primitive=primitive,
            before_state_id=session.current_state.state_id,
            after_state_id=after_state.state_id,
            operation=operation,
            deltas=tuple(deltas),
            produced_evidence_ids=tuple(evidence_ids),
            produced_candidate_ids=tuple(candidate_ids),
            contract_protocol_id=contract_protocol_id,
            parent_transition_id=parent_transition_id,
            validation_passed=validation_passed,
            validation_message=validation_message,
            experiment_id=session.experiment_id,
            condition=session.condition,
            tags=tuple(tags),
            timestamp=ts,
            metadata=metadata or {},
        )
        updated = session.model_copy(update={"transitions": session.transitions + (transition,)})
        return updated.with_cache(after_state)


class ObserverManager:
    def __init__(self, observers: Sequence[IObserver] | None = None):
        self._observers: List[IObserver] = list(
            observers or [LatencyObserver(), MetricObserver()]
        )

    def register(self, observer: IObserver) -> None:
        self._observers.append(observer)

    async def notify(self, session: ReasoningSession, transition: ReasoningTransition) -> ReasoningSession:
        records = []
        for obs in self._observers:
            rec = await obs.observe(transition, session.current_state)
            records.append(rec)
        return session.model_copy(
            update={"observer_records": session.observer_records + tuple(records)}
        )


class ReplayEngine:
    """History = Truth。Current は Cache。"""

    def rebuild(self, session: ReasoningSession, up_to: Optional[int] = None) -> ReasoningState:
        # v1.2: 完全な delta 再適用は未実装。cache を優先し、履歴長の整合を保証。
        if session._current_cache is not None and (
            up_to is None or up_to >= len(session.transitions) - 1
        ):
            return session._current_cache
        return session.initial_state

    def replay(self, session: ReasoningSession, up_to: Optional[int] = None) -> ReasoningSession:
        return session.with_cache(self.rebuild(session, up_to))

    def slice(self, session: ReasoningSession, start: int, end: int) -> ReasoningSession:
        """Transition 列の部分列を新しい session として切り出す（実験比較用）。"""
        if start < 0 or end > len(session.transitions) or start >= end:
            raise IndexError("invalid slice range")
        sliced = session.transitions[start:end]
        new_s = session.model_copy(
            update={
                "session_id": f"slice_{uuid.uuid4().hex[:8]}",
                "transitions": sliced,
            }
        )
        return new_s.with_cache(None)

    def fork(self, session: ReasoningSession, at: int, new_condition: str = "") -> ReasoningSession:
        if at < 0 or at >= len(session.transitions):
            raise IndexError("fork index out of range")
        truncated = session.transitions[: at + 1]
        forked = session.model_copy(
            update={
                "session_id": f"fork_{uuid.uuid4().hex[:8]}",
                "transitions": truncated,
                "condition": new_condition or f"{session.condition}+fork@{at}",
            }
        )
        return forked.with_cache(None)


class ContractResolver:
    def __init__(self, contracts: Sequence[Contract] = ()):
        self._by_id: Dict[str, Contract] = {c.protocol_id: c for c in contracts}

    def resolve(self, protocol_id: str) -> Optional[Contract]:
        return self._by_id.get(protocol_id)

    def register(self, contract: Contract) -> None:
        self._by_id[contract.protocol_id] = contract


# =============================================================================
# 12. LRPRuntime (thin coordinator)
# =============================================================================

class LRPRuntime:
    """
    Coordinator only.
    推論を実行しない。遷移を記録し、定量し、再生する。
    """

    def __init__(
        self,
        observers: Sequence[IObserver] | None = None,
        contracts: Sequence[Contract] = (),
        seed: int = 0,
        clock: Optional[DeterministicClock] = None,
    ):
        self.seed = seed
        self.id_factory = DeterministicIDFactory(seed=seed)
        self.clock = clock or DeterministicClock()
        self.transition_manager = TransitionManager()
        self.observer_manager = ObserverManager(observers)
        self.replay_engine = ReplayEngine()
        self.contract_resolver = ContractResolver(contracts)

    def create_session(
        self,
        problem_id: str,
        capabilities: Sequence[Capability] = (),
        contracts: Sequence[Contract] = (),
        initial_context: Optional[ContextGraph] = None,
        experiment_id: str = "",
        condition: str = "",
    ) -> ReasoningSession:
        sid = self.id_factory("state")
        initial = ReasoningState(
            state_id=sid,
            context=initial_context or ContextGraph(),
            created_at=self.clock(),
        )
        for c in contracts:
            self.contract_resolver.register(c)
        session = ReasoningSession(
            session_id=self.id_factory("session"),
            problem_id=problem_id,
            initial_state=initial,
            capabilities=tuple(capabilities),
            contracts=tuple(contracts),
            experiment_id=experiment_id or f"exp_{self.seed}",
            seed=self.seed,
            condition=condition,
            created_at=self.clock(),
        )
        return session.with_cache(initial)

    async def transition(
        self,
        session: ReasoningSession,
        primitive: ReasoningPrimitive,
        operation: str,
        *,
        context_updates: Sequence[ContextNode] = (),
        new_evidence: Sequence[Evidence] = (),
        new_candidates: Sequence[Candidate] = (),
        deltas: Sequence[PLPDelta] = (),
        contract_protocol_id: Optional[str] = None,
        parent_transition_id: Optional[str] = None,
        notes: str = "",
        validation_passed: bool = True,
        validation_message: str = "",
        tags: Sequence[str] = (),
    ) -> ReasoningSession:
        prev = session.current_state
        ctx = prev.context
        for node in context_updates:
            ctx = ctx.add(node)

        after = ReasoningState(
            state_id=self.id_factory("state"),
            context=ctx,
            evidence=prev.evidence + tuple(new_evidence),
            candidates=prev.candidates + tuple(new_candidates),
            last_primitive=primitive,
            step_count=prev.step_count + 1,
            notes=notes or operation,
            created_at=self.clock(),
        )

        auto_deltas: List[PLPDelta] = list(deltas)
        for ev in new_evidence:
            auto_deltas.append(
                PLPDelta(kind=DeltaKind.EVIDENCE, payload=ev.model_dump(), target_id=ev.evidence_id)
            )
        for cand in new_candidates:
            auto_deltas.append(
                PLPDelta(kind=DeltaKind.CANDIDATE, payload=cand.model_dump(), target_id=cand.candidate_id)
            )
        if context_updates:
            auto_deltas.append(
                PLPDelta(kind=DeltaKind.CONTEXT, payload=[n.model_dump() for n in context_updates])
            )
        auto_deltas.append(
            PLPDelta(
                kind=DeltaKind.PRIMITIVE,
                payload={"primitive": primitive.value, "operation": operation},
            )
        )

        session = self.transition_manager.apply(
            session,
            primitive=primitive,
            operation=operation,
            after_state=after,
            deltas=auto_deltas,
            evidence_ids=[e.evidence_id for e in new_evidence],
            candidate_ids=[c.candidate_id for c in new_candidates],
            contract_protocol_id=contract_protocol_id,
            parent_transition_id=parent_transition_id,
            validation_passed=validation_passed,
            validation_message=validation_message,
            tags=tags,
            id_factory=self.id_factory,
            clock=self.clock,
        )
        last_tr = session.transitions[-1]
        session = await self.observer_manager.notify(session, last_tr)
        return session

    def replay(self, session: ReasoningSession, up_to: Optional[int] = None) -> ReasoningSession:
        return self.replay_engine.replay(session, up_to)

    def fork(self, session: ReasoningSession, at: int, new_condition: str = "") -> ReasoningSession:
        return self.replay_engine.fork(session, at, new_condition)

    def slice(self, session: ReasoningSession, start: int, end: int) -> ReasoningSession:
        return self.replay_engine.slice(session, start, end)

    def metrics(self, session: ReasoningSession) -> SessionMetrics:
        return compute_session_metrics(session)

    def summary(self, session: ReasoningSession) -> str:
        return paper_summary(session)

    def snapshot(self, session: ReasoningSession) -> Dict[str, Any]:
        return session.model_dump()


# =============================================================================
# Public API
# =============================================================================

__all__ = [
    "PHILOSOPHY",
    "PRIMITIVE_CONTRACT",
    "ReasoningPrimitive",
    "EvidenceKind",
    "Evidence",
    "ContextNode",
    "ContextGraph",
    "Capability",
    "Contract",
    "Candidate",
    "DeltaKind",
    "PLPDelta",
    "ReasoningTransition",
    "ReasoningState",
    "ObserverRecord",
    "ReasoningSession",
    "SessionMetrics",
    "compute_session_metrics",
    "paper_summary",
    "DeterministicClock",
    "DeterministicIDFactory",
    "IObserver",
    "LatencyObserver",
    "MetricObserver",
    "TransitionManager",
    "ObserverManager",
    "ReplayEngine",
    "ContractResolver",
    "LRPRuntime",
]


# =============================================================================
# Research demo — multi-condition style
# =============================================================================

async def _run_condition(seed: int, condition: str) -> ReasoningSession:
    runtime = LRPRuntime(seed=seed)
    ctx = ContextGraph().add(
        ContextNode(
            node_id=runtime.id_factory("ctx"),
            kind="problem",
            payload={"goal": "reduce temperature gap", "target": 25.0},
        )
    )
    session = runtime.create_session(
        problem_id="temp_gap",
        initial_context=ctx,
        experiment_id="lrp_v12_demo",
        condition=condition,
    )

    ev_obs = Evidence(
        evidence_id=runtime.id_factory("ev"),
        kind=EvidenceKind.OBSERVED,
        payload={"temperature": 29.2},
        source="sensor",
        confidence=0.95,
    )
    session = await runtime.transition(
        session, ReasoningPrimitive.OBSERVE, "read temperature",
        new_evidence=[ev_obs], tags=("sensor",),
    )

    session = await runtime.transition(
        session, ReasoningPrimitive.HYPOTHESIS, "positive gap decreasing",
        tags=("hypothesis",),
    )

    # condition による ablation 風の差
    conf = 0.8 if condition != "ablation_low_conf" else 0.4
    ev_inf = Evidence(
        evidence_id=runtime.id_factory("ev"),
        kind=EvidenceKind.INFERENCE,
        payload={"projected": 27.6},
        source="model",
        confidence=conf,
        derived_from=(ev_obs.evidence_id,),
    )
    session = await runtime.transition(
        session, ReasoningPrimitive.INFERENCE, "project next temperature",
        new_evidence=[ev_inf], tags=("inference",),
    )

    val_ok = condition != "ablation_fail_validation"
    session = await runtime.transition(
        session, ReasoningPrimitive.VALIDATION, "consistency check",
        validation_passed=val_ok,
        validation_message="" if val_ok else "projection diverges",
        tags=("validation",),
    )

    cand = Candidate(
        candidate_id=runtime.id_factory("cand"),
        description="accept cooling trajectory",
        score=0.82 if val_ok else 0.3,
        confidence=0.78 if val_ok else 0.35,
        supporting_evidence_ids=(ev_obs.evidence_id, ev_inf.evidence_id),
    )
    session = await runtime.transition(
        session, ReasoningPrimitive.COMMIT, "emit candidate for DCK",
        new_candidates=[cand], tags=("commit", "to_dck"),
    )
    return session


async def _demo() -> None:
    print("=== LRP v1.2-research — Reasoning Transition Protocol ===")
    print("Philosophy:", " / ".join(PHILOSOPHY[:4]), "...")
    print()

    conditions = ["baseline", "ablation_low_conf", "ablation_fail_validation"]
    results: List[Tuple[str, SessionMetrics]] = []

    for i, cond in enumerate(conditions):
        session = await _run_condition(seed=100 + i, condition=cond)
        m = compute_session_metrics(session)
        results.append((cond, m))
        print(f"--- condition: {cond} (seed={100+i}) ---")
        print(paper_summary(session))
        print()

    print("=== Cross-condition comparison (researcher view) ===")
    print(f"{'condition':<28} {'n_tr':>4} {'n_ev':>4} {'val_rate':>8} {'mean_ev_c':>9} {'n_cand':>6}")
    for cond, m in results:
        print(
            f"{cond:<28} {m.n_transitions:>4} {m.n_evidence:>4} "
            f"{m.validation_pass_rate:>8.3f} {m.mean_evidence_confidence:>9.3f} {m.n_candidates:>6}"
        )


if __name__ == "__main__":
    import asyncio
    asyncio.run(_demo())

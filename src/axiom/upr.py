"""
Universal Protocol Runtime (UPR) - v1.2 Final Specification
============================================================

Core Principles:
1. Total Side-Effect Encapsulation (Clock, Thread-Safe IdGenerator)
2. DomainEvent -> EngineEvent Conversion (Stage Context Isolation)
3. Deep Immutable Extensions via Declarative ExtensionOps
4. Separation of PipelineDefinition and Stateless Navigation
"""

from __future__ import annotations

import asyncio
import copy
import dataclasses
import itertools
import threading
import traceback
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Protocol, runtime_checkable


# ===========================================================================
# 1. Thread-Safe Provider Interfaces (Clock & IdGenerator)
# ===========================================================================

@runtime_checkable
class Clock(Protocol):
    def now_iso(self) -> str: ...
    async def sleep(self, seconds: float) -> None: ...


class SystemClock:
    def now_iso(self) -> str:
        from datetime import datetime, timezone
        return datetime.now(timezone.utc).isoformat()

    async def sleep(self, seconds: float) -> None:
        await asyncio.sleep(seconds)


class VirtualClock:
    def __init__(self, start_iso: str = "2026-01-01T00:00:00+00:00") -> None:
        from datetime import datetime, timezone
        self.current = datetime.fromisoformat(start_iso)

    def now_iso(self) -> str:
        return self.current.isoformat()

    async def sleep(self, seconds: float) -> None:
        from datetime import datetime, timezone
        self.current = datetime.fromtimestamp(self.current.timestamp() + seconds, tz=timezone.utc)


@runtime_checkable
class IdGenerator(Protocol):
    def generate(self) -> str: ...


class UUIDv4Generator:
    def generate(self) -> str:
        import uuid
        return str(uuid.uuid4())


class ThreadSafeSequentialIdGenerator:
    """スレッドセーフ・非同期セーフな連番 ID 発行器。"""

    def __init__(self, prefix: str = "id") -> None:
        self.prefix = prefix
        self._counter = itertools.count(1)
        self._lock = threading.Lock()

    def generate(self) -> str:
        with self._lock:
            val = next(self._counter)
        return f"{self.prefix}-{val:06d}"


# ===========================================================================
# 2. Deep Immutable Extensions & Extension Operations
# ===========================================================================

class ExtensionOpType(str, Enum):
    SET = "set"
    MERGE = "merge"
    DELETE = "delete"


@dataclass(frozen=True)
class ExtensionOp:
    """拡張領域に対する宣言的操作（差分命令）。"""
    namespace: str
    key: str
    op_type: ExtensionOpType = ExtensionOpType.SET
    value: Any = None


@dataclass(frozen=True)
class NamespacedExtensions:
    store: dict[str, dict[str, Any]] = field(default_factory=dict)

    def get(self, namespace: str, key: str, default: Any = None) -> Any:
        val = self.store.get(namespace, {}).get(key, default)
        return copy.deepcopy(val)

    def apply_ops(self, ops: tuple[ExtensionOp, ...]) -> NamespacedExtensions:
        new_store = copy.deepcopy(self.store)
        for op in ops:
            ns_dict = new_store.setdefault(op.namespace, {})
            if op.op_type == ExtensionOpType.SET:
                ns_dict[op.key] = copy.deepcopy(op.value)
            elif op.op_type == ExtensionOpType.MERGE:
                existing = ns_dict.get(op.key)
                if isinstance(existing, dict) and isinstance(op.value, dict):
                    merged = copy.deepcopy(existing)
                    merged.update(copy.deepcopy(op.value))
                    ns_dict[op.key] = merged
                else:
                    ns_dict[op.key] = copy.deepcopy(op.value)
            elif op.op_type == ExtensionOpType.DELETE:
                ns_dict.pop(op.key, None)

        return NamespacedExtensions(store=new_store)


# ===========================================================================
# 3. Context & Event Models (Domain vs Engine Events)
# ===========================================================================

class EngineStatus(str, Enum):
    IDLE = "idle"
    RUNNING = "running"
    FINISHED = "finished"
    ERROR = "error"
    ABORTED = "aborted"


@dataclass(frozen=True)
class RuntimeMetadata:
    run_id: str
    created_at: str
    engine_version: str = "1.2.0"


@dataclass(frozen=True)
class StateContext:
    payload: dict[str, Any] = field(default_factory=dict)
    sequence_id: str = ""
    revision: int = 0


@dataclass(frozen=True)
class ProtocolContext:
    metadata: RuntimeMetadata
    state: StateContext
    extensions: NamespacedExtensions = field(default_factory=NamespacedExtensions)

    def with_state(self, new_payload: dict[str, Any], new_seq_id: str) -> ProtocolContext:
        new_state = StateContext(
            payload=copy.deepcopy(new_payload),
            sequence_id=new_seq_id,
            revision=self.state.revision + 1,
        )
        return dataclasses.replace(self, state=new_state)


@dataclass(frozen=True)
class EngineContext:
    status: EngineStatus = EngineStatus.IDLE
    step_count: int = 0


@dataclass(frozen=True)
class RuntimeContextSnapshot:
    engine: EngineContext
    protocol: ProtocolContext


@dataclass(frozen=True)
class DomainEvent:
    """Stage が出力する極小ドメインイベント（時間・IDのメタデータを持たない）。"""
    event_type: str
    payload: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class EngineEvent:
    """Runtime が DomainEvent を外包し、タイムスタンプ・ID を付与した標準イベント。"""
    event_id: str
    timestamp: str
    event_type: str
    payload: dict[str, Any] = field(default_factory=dict)


# ===========================================================================
# 4. Core Protocols & Pipeline Isolation
# ===========================================================================

@dataclass(frozen=True)
class StageResult:
    payload: dict[str, Any]
    events: tuple[DomainEvent, ...] = ()
    extension_ops: tuple[ExtensionOp, ...] = ()


@runtime_checkable
class Stage(Protocol):
    name: str
    async def execute(self, context: ProtocolContext) -> StageResult: ...


@dataclass(frozen=True)
class PipelineDefinition:
    """パイプラインの静的定義（トポロジーデータ）。"""
    name: str
    stages: tuple[Stage, ...]


@runtime_checkable
class Pipeline(Protocol):
    """ProtocolContext から次の Stage を算出する純粋ナビゲーター。"""
    name: str
    def next_stage(self, context: ProtocolContext) -> Stage | None: ...


class LinearPipeline:
    """順次実行を行う標準の Pipeline ナビゲーター実装。"""

    def __init__(self, definition: PipelineDefinition) -> None:
        self.definition = definition

    @property
    def name(self) -> str:
        return self.definition.name

    def next_stage(self, context: ProtocolContext) -> Stage | None:
        idx = context.state.revision
        if idx < len(self.definition.stages):
            return self.definition.stages[idx]
        return None


# ===========================================================================
# 5. Sidecar Interfaces
# ===========================================================================

@runtime_checkable
class EventSink(Protocol):
    async def emit(self, event: EngineEvent) -> None: ...


class ConsoleEventSink:
    async def emit(self, event: EngineEvent) -> None:
        print(f"[{event.timestamp}] [EVENT:{event.event_id}] {event.event_type:<20} | {event.payload}")


@runtime_checkable
class HistoryRecorder(Protocol):
    async def record(self, snapshot: RuntimeContextSnapshot) -> None: ...


class MemoryHistoryRecorder:
    def __init__(self) -> None:
        self.snapshots: list[RuntimeContextSnapshot] = []

    async def record(self, snapshot: RuntimeContextSnapshot) -> None:
        self.snapshots.append(snapshot)


# ===========================================================================
# 6. Universal Protocol Runtime Core (UPR v1.2)
# ===========================================================================

class UniversalProtocolRuntime:
    def __init__(
        self,
        clock: Clock | None = None,
        id_generator: IdGenerator | None = None,
        event_sink: EventSink | None = None,
        history_recorder: HistoryRecorder | None = None,
        max_steps: int = 1000,
    ) -> None:
        self.clock = clock or SystemClock()
        self.id_generator = id_generator or UUIDv4Generator()
        self.event_sink = event_sink
        self.history_recorder = history_recorder
        self.max_steps = max_steps

    async def create_protocol_context(self, initial_payload: dict[str, Any]) -> ProtocolContext:
        run_id = self.id_generator.generate()
        created_at = self.clock.now_iso()
        seq_id = self.id_generator.generate()

        return ProtocolContext(
            metadata=RuntimeMetadata(run_id=run_id, created_at=created_at),
            state=StateContext(payload=copy.deepcopy(initial_payload), sequence_id=seq_id, revision=0),
        )

    async def run(
        self,
        pipeline: Pipeline,
        protocol_context: ProtocolContext,
    ) -> RuntimeContextSnapshot:
        engine_ctx = EngineContext(status=EngineStatus.RUNNING, step_count=0)

        await self._emit("engine_started", {"pipeline": pipeline.name})
        await self._record(engine_ctx, protocol_context)

        current_proto_ctx = protocol_context

        while True:
            # 1. Watchdog
            if engine_ctx.step_count >= self.max_steps:
                engine_ctx = dataclasses.replace(engine_ctx, status=EngineStatus.ABORTED)
                await self._emit("engine_aborted", {"reason": "max_steps_exceeded"})
                await self._record(engine_ctx, current_proto_ctx)
                break

            # 2. Stateless Stage Navigation
            stage = pipeline.next_stage(current_proto_ctx)
            if stage is None:
                engine_ctx = dataclasses.replace(engine_ctx, status=EngineStatus.FINISHED)
                await self._emit("engine_finished", {"total_steps": engine_ctx.step_count})
                await self._record(engine_ctx, current_proto_ctx)
                break

            engine_ctx = dataclasses.replace(engine_ctx, step_count=engine_ctx.step_count + 1)

            # 3. Stage Execution & State Transition
            try:
                result = await stage.execute(current_proto_ctx)

                next_seq_id = self.id_generator.generate()
                new_proto_ctx = current_proto_ctx.with_state(
                    new_payload=result.payload,
                    new_seq_id=next_seq_id,
                )

                # ExtensionOps の不変適用
                if result.extension_ops:
                    new_ext = new_proto_ctx.extensions.apply_ops(result.extension_ops)
                    new_proto_ctx = dataclasses.replace(new_proto_ctx, extensions=new_ext)

                current_proto_ctx = new_proto_ctx

                # DomainEvent -> EngineEvent 変換と送出
                for domain_event in result.events:
                    await self._emit(domain_event.event_type, domain_event.payload)

            except Exception as exc:
                engine_ctx = dataclasses.replace(engine_ctx, status=EngineStatus.ERROR)
                await self._emit("engine_error", {
                    "stage": stage.name,
                    "exception": type(exc).__name__,
                    "message": str(exc),
                    "traceback": traceback.format_exc(),
                })
                await self._record(engine_ctx, current_proto_ctx)
                break

            await self._record(engine_ctx, current_proto_ctx)

        return RuntimeContextSnapshot(engine=engine_ctx, protocol=current_proto_ctx)

    async def _emit(self, event_type: str, payload: dict[str, Any]) -> None:
        if self.event_sink:
            event = EngineEvent(
                event_id=self.id_generator.generate(),
                timestamp=self.clock.now_iso(),
                event_type=event_type,
                payload=payload,
            )
            try:
                await self.event_sink.emit(event)
            except Exception:
                pass  # Fault Isolation

    async def _record(self, engine: EngineContext, protocol: ProtocolContext) -> None:
        if self.history_recorder:
            try:
                await self.history_recorder.record(
                    RuntimeContextSnapshot(engine=engine, protocol=protocol)
                )
            except Exception:
                pass  # Fault Isolation


# ===========================================================================
# Demo & Verification
# ===========================================================================

class PureCompileStage:
    name = "pure.compile"

    async def execute(self, context: ProtocolContext) -> StageResult:
        new_payload = {**context.state.payload, "compiled": True}

        # Stage は DomainEvent と ExtensionOp のみを記述（副作用・時刻・ID非依存）
        domain_event = DomainEvent("compilation_succeeded", {"module": "core"})
        ext_op = ExtensionOp(
            namespace="axiom",
            key="config",
            op_type=ExtensionOpType.MERGE,
            value={"version": "1.2.0", "target": "wasm"},
        )

        return StageResult(
            payload=new_payload,
            events=(domain_event,),
            extension_ops=(ext_op,),
        )


async def main() -> None:
    print("=" * 70)
    print("Universal Protocol Runtime (UPR) v1.2 Standard Architecture")
    print("=" * 70)

    clock = VirtualClock()
    id_gen = ThreadSafeSequentialIdGenerator(prefix="seq")
    event_sink = ConsoleEventSink()
    history = MemoryHistoryRecorder()

    runtime = UniversalProtocolRuntime(
        clock=clock,
        id_generator=id_gen,
        event_sink=event_sink,
        history_recorder=history,
    )

    proto_ctx = await runtime.create_protocol_context({"raw": "Specification Complete"})

    definition = PipelineDefinition(
        name="VerificationPipeline",
        stages=(PureCompileStage(),),
    )
    pipeline = LinearPipeline(definition)

    final_snapshot = await runtime.run(pipeline, proto_ctx)

    print("\n--- Final Snapshot Summary ---")
    print(f"Status      : {final_snapshot.engine.status.value}")
    print(f"Total Steps : {final_snapshot.engine.step_count}")
    print(f"Revision    : {final_snapshot.protocol.state.revision}")
    print(f"Ext Config  : {final_snapshot.protocol.extensions.get('axiom', 'config')}")


if __name__ == "__main__":
    asyncio.run(main())

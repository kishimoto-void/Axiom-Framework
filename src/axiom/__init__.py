"""
Axiom Framework — Core Runtime Package

Universal Protocol Runtime (UPR) v1.2 を基盤とした
言語非依存・副作用完全隔離の決定論的状態遷移エンジン。
"""

from .upr import (
    # Core Runtime
    UniversalProtocolRuntime,
    # Contexts
    ProtocolContext,
    RuntimeContextSnapshot,
    EngineContext,
    StateContext,
    RuntimeMetadata,
    EngineStatus,
    # Events
    DomainEvent,
    EngineEvent,
    # Stages & Pipeline
    Stage,
    StageResult,
    Pipeline,
    PipelineDefinition,
    LinearPipeline,
    # Extensions
    ExtensionOp,
    ExtensionOpType,
    NamespacedExtensions,
    # Providers
    Clock,
    SystemClock,
    VirtualClock,
    IdGenerator,
    UUIDv4Generator,
    ThreadSafeSequentialIdGenerator,
    # Sidecars
    EventSink,
    ConsoleEventSink,
    HistoryRecorder,
    MemoryHistoryRecorder,
)

__version__ = "0.1.0-upr1.2"
__all__ = [
    "UniversalProtocolRuntime",
    "ProtocolContext",
    "RuntimeContextSnapshot",
    "EngineContext",
    "StateContext",
    "RuntimeMetadata",
    "EngineStatus",
    "DomainEvent",
    "EngineEvent",
    "Stage",
    "StageResult",
    "Pipeline",
    "PipelineDefinition",
    "LinearPipeline",
    "ExtensionOp",
    "ExtensionOpType",
    "NamespacedExtensions",
    "Clock",
    "SystemClock",
    "VirtualClock",
    "IdGenerator",
    "UUIDv4Generator",
    "ThreadSafeSequentialIdGenerator",
    "EventSink",
    "ConsoleEventSink",
    "HistoryRecorder",
    "MemoryHistoryRecorder",
]

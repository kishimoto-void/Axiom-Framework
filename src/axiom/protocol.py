"""
Axiom Framework Common Protocol Specification v2.7
Universal Axiomatic Protocol Contract & Deterministic Runtime Guard.

Algebraically closed, fully reversible, and mathematically immutable protocol layer.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from collections.abc import Mapping
from dataclasses import dataclass
from enum import Enum
import hashlib
import math
from typing import Any, Optional, Tuple, Union
import unicodedata
import weakref

MAX_RECURSION_DEPTH = 64


# ============================================================ #
# 1. Axiomatic Closed Type System (Algebraic State Space)
# ============================================================ #

def _strict_eq(a: Any, b: Any) -> bool:
    """Recursively validates type identity and content equality without implicit coercions."""
    if type(a) is not type(b):
        return False
    if isinstance(a, tuple):
        if len(a) != len(b):
            return False
        return all(_strict_eq(x, y) for x, y in zip(a, b))
    if isinstance(a, FrozenDict):
        return a == b
    return a == b


def _get_strict_type_key(val: Any) -> Any:
    """Generates a structural key preserving exact type identities recursively for hashing."""
    if isinstance(val, float):
        return (float, 0.0 if val == 0.0 else val)
    if isinstance(val, tuple):
        return (tuple, tuple(_get_strict_type_key(x) for x in val))
    if isinstance(val, FrozenDict):
        items_key = tuple(
            sorted(
                ((k, _get_strict_type_key(v)) for k, v in val._store.items()),
                key=lambda item: item[0].encode("utf-8"),
            )
        )
        return (FrozenDict, items_key)
    return (type(val), val)


class FrozenDict(Mapping[str, "AxiomValue"]):
    """
    Immutable, hashable dictionary with strict type and content-based equality.
    
    Note on Hashing:
        __hash__() produces an in-process hash value for Python set/dict usage.
        For deterministic, environment-invariant cryptographic proofs, use ProtocolSnapshot.digest.
    """

    __slots__ = ("_store", "_hash")

    def __init__(self, mapping: Mapping[str, Any] | None = None, **kwargs: Any):
        data = dict(mapping or {}, **kwargs)
        normalized: dict[str, AxiomValue] = {}
        for k, v in data.items():
            if not isinstance(k, str):
                raise TypeError(f"FrozenDict key must be str, got {type(k).__name__}")
            norm_key = unicodedata.normalize("NFC", k)
            normalized[norm_key] = _sanitize_value(v)
        self._store = normalized
        self._hash: Optional[int] = None

    @classmethod
    def _from_validated(cls, store: dict[str, AxiomValue]) -> FrozenDict:
        """Internal constructor for pre-validated, pre-normalized mappings with defensive copy."""
        obj = cls.__new__(cls)
        obj._store = dict(store)  # Defensive copy to eliminate internal mutability leaks
        obj._hash = None
        return obj

    def __getitem__(self, key: str) -> AxiomValue:
        return self._store[unicodedata.normalize("NFC", key)]

    def __len__(self) -> int:
        return len(self._store)

    def __iter__(self):
        return iter(self._store)

    def __eq__(self, other: Any) -> bool:
        if self is other:
            return True
        if type(other) is not FrozenDict:
            return False
        if len(self._store) != len(other._store):
            return False
        for k, v in self._store.items():
            if k not in other._store:
                return False
            if not _strict_eq(v, other._store[k]):
                return False
        return True

    def __hash__(self) -> int:
        if self._hash is None:
            items_tuple = tuple(
                sorted(
                    ((k, _get_strict_type_key(v)) for k, v in self._store.items()),
                    key=lambda item: item[0].encode("utf-8"),
                )
            )
            self._hash = hash((FrozenDict, items_tuple))
        return self._hash

    def __repr__(self) -> str:
        return f"FrozenDict({self._store!r})"


AxiomPrimitive = Union[None, bool, int, float, str]
AxiomValue = Union[AxiomPrimitive, Tuple["AxiomValue", ...], FrozenDict]


def _sanitize_value(
    val: Any, depth: int = 0, ancestor_ids: set[int] | None = None
) -> AxiomValue:
    """Recursively validates type constraints and enforces stack-based cycle detection."""
    if depth > MAX_RECURSION_DEPTH:
        raise AxiomProtocolError(
            AxiomErrorPayload(
                code=ProtocolErrorCode.MAX_DEPTH_EXCEEDED,
                details=FrozenDict({"max_depth": MAX_RECURSION_DEPTH}),
            )
        )

    if ancestor_ids is None:
        ancestor_ids = set()

    is_container = isinstance(val, (dict, list, tuple, Mapping, FrozenDict))
    val_id = id(val)

    if is_container:
        if val_id in ancestor_ids:
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.CIRCULAR_REFERENCE_DETECTED,
                    details=FrozenDict({"type": type(val).__name__}),
                )
            )
        ancestor_ids.add(val_id)

    try:
        if val is None or isinstance(val, bool) or isinstance(val, int):
            return val

        if isinstance(val, float):
            if math.isnan(val) or math.isinf(val):
                raise AxiomProtocolError(
                    AxiomErrorPayload(
                        code=ProtocolErrorCode.INVALID_PAYLOAD,
                        details=FrozenDict({"reason": "Non-deterministic float (NaN/Inf)"}),
                    )
                )
            return 0.0 if val == 0.0 else val

        if isinstance(val, str):
            return unicodedata.normalize("NFC", val)

        if isinstance(val, (tuple, list)):
            return tuple(
                _sanitize_value(item, depth + 1, ancestor_ids) for item in val
            )

        if isinstance(val, (dict, Mapping)):
            sanitized_dict: dict[str, AxiomValue] = {}
            for k, v in val.items():
                if not isinstance(k, str):
                    raise AxiomProtocolError(
                        AxiomErrorPayload(
                            code=ProtocolErrorCode.INVALID_PAYLOAD,
                            details=FrozenDict({"reason": f"Key must be str, got {type(k).__name__}"}),
                        )
                    )
                norm_k = unicodedata.normalize("NFC", k)
                sanitized_dict[norm_k] = _sanitize_value(v, depth + 1, ancestor_ids)
            return FrozenDict._from_validated(sanitized_dict)

        raise AxiomProtocolError(
            AxiomErrorPayload(
                code=ProtocolErrorCode.INVALID_PAYLOAD,
                details=FrozenDict({"unsupported_type": type(val).__name__}),
            )
        )
    finally:
        if is_container:
            ancestor_ids.remove(val_id)


# ============================================================ #
# 2. Reversible Length-Prefixed Canonical Serializer Engine
# ============================================================ #

def _parse_canonical_int(raw_bytes: bytes) -> int:
    """Parses ASCII integer bytes strictly enforcing canonical representation rules."""
    if not raw_bytes:
        raise ValueError("Empty integer byte string in stream")
    val_str = raw_bytes.decode("ascii")
    if val_str == "0":
        return 0
    if val_str.startswith("-"):
        if len(val_str) <= 1 or val_str[1] == "0" or not val_str[1:].isdigit():
            raise ValueError(f"Non-canonical negative integer format: {val_str!r}")
    else:
        if val_str.startswith("0") or not val_str.isdigit():
            raise ValueError(f"Non-canonical positive integer format: {val_str!r}")
    return int(val_str)


def _parse_canonical_uint(raw_bytes: bytes) -> int:
    """Parses non-negative ASCII integer bytes strictly for length and count prefixes."""
    val = _parse_canonical_int(raw_bytes)
    if val < 0:
        raise ValueError(f"Length/count prefix cannot be negative: {val}")
    return val


class ByteReader:
    """Helper for reading length-prefixed bytes during deserialization."""

    def __init__(self, data: bytes):
        self._data = data
        self._offset = 0

    @property
    def has_more(self) -> bool:
        return self._offset < len(self._data)

    def read_exact(self, length: int) -> bytes:
        if self._offset + length > len(self._data):
            raise ValueError("Unexpected EOF in canonical byte stream")
        res = self._data[self._offset : self._offset + length]
        self._offset += length
        return res

    def read_until(self, delimiter: bytes) -> bytes:
        idx = self._data.find(delimiter, self._offset)
        if idx == -1:
            raise ValueError(f"Delimiter {delimiter!r} not found in canonical byte stream")
        res = self._data[self._offset : idx]
        self._offset = idx + len(delimiter)
        return res


class CanonicalSerializer:
    """Bijective, environment-invariant canonical byte serializer and deserializer."""

    @classmethod
    def serialize_to_bytes(cls, value: AxiomValue) -> bytes:
        buf = bytearray()
        cls._encode(value, buf)
        return bytes(buf)

    @classmethod
    def deserialize_from_bytes(cls, data: bytes) -> AxiomValue:
        reader = ByteReader(data)
        val = cls._decode(reader)
        if reader.has_more:
            raise ValueError("Trailing unparsed bytes found after deserialization")
        return val

    @classmethod
    def _encode(cls, val: AxiomValue, buf: bytearray) -> None:
        if val is None:
            buf.extend(b"Z")
        elif isinstance(val, bool):
            buf.extend(b"B1" if val else b"B0")
        elif isinstance(val, int):
            encoded = str(val).encode("ascii")
            buf.extend(f"I{len(encoded)}:".encode("ascii"))
            buf.extend(encoded)
        elif isinstance(val, float):
            hex_str = val.hex().encode("ascii")
            buf.extend(f"F{len(hex_str)}:".encode("ascii"))
            buf.extend(hex_str)
        elif isinstance(val, str):
            encoded = val.encode("utf-8")
            buf.extend(f"S{len(encoded)}:".encode("ascii"))
            buf.extend(encoded)
        elif isinstance(val, tuple):
            buf.extend(f"T{len(val)}:".encode("ascii"))
            for item in val:
                cls._encode(item, buf)
        elif isinstance(val, FrozenDict):
            buf.extend(f"D{len(val)}:".encode("ascii"))
            sorted_keys = sorted(val.keys(), key=lambda k: k.encode("utf-8"))
            for k in sorted_keys:
                cls._encode(k, buf)
                cls._encode(val[k], buf)
        else:
            raise TypeError(f"Unencodable type in CanonicalSerializer: {type(val)}")

    @classmethod
    def _decode(cls, reader: ByteReader) -> AxiomValue:
        tag = reader.read_exact(1)
        if tag == b"Z":
            return None
        if tag == b"B":
            flag = reader.read_exact(1)
            if flag == b"1":
                return True
            if flag == b"0":
                return False
            raise ValueError(f"Invalid boolean flag: {flag!r}")
        if tag == b"I":
            length_bytes = reader.read_until(b":")
            length = _parse_canonical_uint(length_bytes)
            val_bytes = reader.read_exact(length)
            return _parse_canonical_int(val_bytes)
        if tag == b"F":
            length_bytes = reader.read_until(b":")
            length = _parse_canonical_uint(length_bytes)
            raw_hex_bytes = reader.read_exact(length)
            raw_hex_str = raw_hex_bytes.decode("ascii")
            val = float.fromhex(raw_hex_str)
            
            if math.isnan(val) or math.isinf(val):
                raise ValueError("Non-deterministic float (NaN/Inf) encountered in byte stream")
                
            norm_val = 0.0 if val == 0.0 else val
            canonical_hex_bytes = norm_val.hex().encode("ascii")
            
            if raw_hex_bytes != canonical_hex_bytes:
                raise ValueError(
                    f"Non-canonical float hex string in stream: expected {canonical_hex_bytes!r}, got {raw_hex_bytes!r}"
                )
            return norm_val
        if tag == b"S":
            length_bytes = reader.read_until(b":")
            length = _parse_canonical_uint(length_bytes)
            raw_bytes = reader.read_exact(length)
            s = raw_bytes.decode("utf-8")
            if unicodedata.normalize("NFC", s) != s or s.encode("utf-8") != raw_bytes:
                raise ValueError("Non-NFC or unnormalized UTF-8 string encountered in stream")
            return s
        if tag == b"T":
            count_bytes = reader.read_until(b":")
            count = _parse_canonical_uint(count_bytes)
            return tuple(cls._decode(reader) for _ in range(count))
        if tag == b"D":
            count_bytes = reader.read_until(b":")
            count = _parse_canonical_uint(count_bytes)
            items: dict[str, AxiomValue] = {}
            last_key_bytes: bytes | None = None
            
            for _ in range(count):
                k = cls._decode(reader)
                if not isinstance(k, str):
                    raise ValueError("Deserialized dictionary key must be a string")
                
                # Double-check NFC normalization for dict keys defensively
                if unicodedata.normalize("NFC", k) != k:
                    raise ValueError(f"Non-NFC normalized dictionary key: {k!r}")
                
                k_bytes = k.encode("utf-8")
                if last_key_bytes is not None:
                    if k_bytes <= last_key_bytes:
                        if k_bytes == last_key_bytes:
                            raise ValueError(f"Duplicate dictionary key encountered: {k}")
                        raise ValueError(f"Non-canonical unsorted dictionary key encountered: {k}")
                last_key_bytes = k_bytes
                
                v = cls._decode(reader)
                items[k] = v
                
            return FrozenDict._from_validated(items)

        raise ValueError(f"Unknown type tag in canonical stream: {tag!r}")


# ============================================================ #
# 3. Machine-Readable Structured Error System
# ============================================================ #

class ProtocolErrorCode(str, Enum):
    MALFORMED_CONTEXT = "ERR_MALFORMED_CONTEXT"
    INVALID_PAYLOAD = "ERR_INVALID_PAYLOAD"
    MALFORMED_VERSION = "ERR_MALFORMED_VERSION"
    VERSION_INCOMPATIBLE = "ERR_VERSION_INCOMPATIBLE"
    CIRCULAR_REFERENCE_DETECTED = "ERR_CIRCULAR_REFERENCE_DETECTED"
    MAX_DEPTH_EXCEEDED = "ERR_MAX_DEPTH_EXCEEDED"
    STATE_TRANSITION_FAILED = "ERR_STATE_TRANSITION_FAILED"
    PROTOCOL_NAME_MISMATCH = "ERR_PROTOCOL_NAME_MISMATCH"
    HASH_VERIFICATION_FAILED = "ERR_HASH_VERIFICATION_FAILED"
    SNAPSHOT_CAPTURE_FAILED = "ERR_SNAPSHOT_CAPTURE_FAILED"
    RESTORE_TRANSACTION_FAILED = "ERR_RESTORE_TRANSACTION_FAILED"
    NOT_INITIALIZED = "ERR_NOT_INITIALIZED"


@dataclass(frozen=True)
class AxiomErrorPayload:
    """Pure machine-readable error payload containing zero unstructured text."""

    code: ProtocolErrorCode
    details: FrozenDict

    def to_canonical_bytes(self) -> bytes:
        return CanonicalSerializer.serialize_to_bytes(
            FrozenDict({"code": self.code.value, "details": self.details})
        )


class AxiomProtocolError(Exception):
    """Base exception for all Axiom protocol operations."""

    def __init__(self, payload: AxiomErrorPayload):
        self.payload = payload
        super().__init__(payload.code.value)

    def __repr__(self) -> str:
        return f"AxiomProtocolError({self.payload.code.value})"


# ============================================================ #
# 4. Algebraic Version Space
# ============================================================ #

@dataclass(frozen=True)
class ProtocolVersion:
    """Strict Version tuple representation."""

    major: int
    minor: int
    patch: int

    @classmethod
    def parse(cls, version_str: str) -> ProtocolVersion:
        parts = version_str.split(".")
        if len(parts) != 3:
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.MALFORMED_VERSION,
                    details=FrozenDict({"raw_version": version_str}),
                )
            )
        for p in parts:
            if not p.isdigit() or (p != "0" and p.startswith("0")):
                raise AxiomProtocolError(
                    AxiomErrorPayload(
                        code=ProtocolErrorCode.MALFORMED_VERSION,
                        details=FrozenDict({"raw_version": version_str}),
                    )
                )
        return cls(int(parts[0]), int(parts[1]), int(parts[2]))

    def is_runtime_compatible_with(self, snapshot_version: ProtocolVersion) -> bool:
        """Algebraic Version Rule: Runtime must match major version and be at least equal in minor version."""
        if self.major != snapshot_version.major:
            return False
        return self.minor >= snapshot_version.minor

    def to_tuple(self) -> Tuple[int, int, int]:
        return (self.major, self.minor, self.patch)

    def __str__(self) -> str:
        return f"{self.major}.{self.minor}.{self.patch}"


# ============================================================ #
# 5. Core Protocol Data Structures
# ============================================================ #

@dataclass(frozen=True)
class ProtocolMeta:
    """Immutable protocol identity."""

    name: str
    version: ProtocolVersion
    capabilities: Tuple[str, ...]

    def __post_init__(self) -> None:
        object.__setattr__(self, "name", unicodedata.normalize("NFC", self.name))
        normalized_caps = tuple(
            sorted(set(unicodedata.normalize("NFC", c) for c in self.capabilities))
        )
        object.__setattr__(self, "capabilities", normalized_caps)


@dataclass(frozen=True)
class ProtocolContext:
    """Runtime execution context issued by UPR."""

    session_id: str
    trace_id: str
    epoch_timestamp_ms: int
    metadata: FrozenDict

    def __post_init__(self) -> None:
        norm_session = unicodedata.normalize("NFC", self.session_id).strip()
        norm_trace = unicodedata.normalize("NFC", self.trace_id).strip()

        if not norm_session:
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.MALFORMED_CONTEXT,
                    details=FrozenDict({"field": "session_id"}),
                )
            )
        if not norm_trace:
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.MALFORMED_CONTEXT,
                    details=FrozenDict({"field": "trace_id"}),
                )
            )
        if self.epoch_timestamp_ms <= 0:
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.MALFORMED_CONTEXT,
                    details=FrozenDict({"field": "epoch_timestamp_ms"}),
                )
            )

        sanitized_meta = _sanitize_value(self.metadata)
        if not isinstance(sanitized_meta, FrozenDict):
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.MALFORMED_CONTEXT,
                    details=FrozenDict({"field": "metadata"}),
                )
            )

        object.__setattr__(self, "session_id", norm_session)
        object.__setattr__(self, "trace_id", norm_trace)
        object.__setattr__(self, "metadata", sanitized_meta)


class ProtocolSnapshot:
    """Deterministic, strictly immutable state snapshot carrying a cryptographic proof."""

    __slots__ = ("_protocol", "_version", "_state", "_digest")

    def __init__(
        self, protocol: str, version: ProtocolVersion, state: Mapping[str, Any]
    ):
        norm_proto = unicodedata.normalize("NFC", protocol)
        frozen_state = FrozenDict(state)

        canonical_bytes = CanonicalSerializer.serialize_to_bytes(
            FrozenDict(
                {
                    "protocol": norm_proto,
                    "version": version.to_tuple(),
                    "state": frozen_state,
                }
            )
        )
        digest = hashlib.sha256(canonical_bytes).hexdigest()

        object.__setattr__(self, "_protocol", norm_proto)
        object.__setattr__(self, "_version", version)
        object.__setattr__(self, "_state", frozen_state)
        object.__setattr__(self, "_digest", digest)

    def __setattr__(self, name: str, value: Any) -> None:
        raise AttributeError("ProtocolSnapshot is strictly immutable")

    def __delattr__(self, name: str) -> None:
        raise AttributeError("ProtocolSnapshot is strictly immutable")

    @property
    def protocol(self) -> str:
        return self._protocol

    @property
    def version(self) -> ProtocolVersion:
        return self._version

    @property
    def state(self) -> FrozenDict:
        return self._state

    @property
    def digest(self) -> str:
        return self._digest

    def verify_integrity(self) -> bool:
        canonical_bytes = CanonicalSerializer.serialize_to_bytes(
            FrozenDict(
                {
                    "protocol": self._protocol,
                    "version": self._version.to_tuple(),
                    "state": self._state,
                }
            )
        )
        expected = hashlib.sha256(canonical_bytes).hexdigest()
        return self._digest == expected


# ============================================================ #
# 6. Pure Protocol Abstract Interface
# ============================================================ #

class AxiomProtocol(ABC):
    """Pure Protocol Contract interface."""

    @property
    @abstractmethod
    def meta(self) -> ProtocolMeta:
        pass

    @abstractmethod
    def initialize(self, context: ProtocolContext) -> None:
        pass

    @abstractmethod
    def validate(
        self, payload: AxiomValue
    ) -> Tuple[bool, Optional[AxiomErrorPayload]]:
        """Validates payload mechanically."""
        pass

    @abstractmethod
    def execute(self, payload: AxiomValue, context: ProtocolContext) -> AxiomValue:
        """Executes a pure state transition."""
        pass

    @abstractmethod
    def snapshot(self) -> ProtocolSnapshot:
        """Exports current state as a deterministic Snapshot."""
        pass

    @abstractmethod
    def restore(self, snapshot: ProtocolSnapshot) -> None:
        """Internal raw state restoration."""
        pass

    @abstractmethod
    def shutdown(self) -> None:
        pass


# ============================================================ #
# 7. Independent Mechanical Protocol Guard (UPR Runtime)
# ============================================================ #

class ProtocolGuard:
    """Isolated runtime guard for protocol management and safe state restoration."""

    _INITIALIZED_REGISTRY: weakref.WeakSet[AxiomProtocol] = weakref.WeakSet()

    @classmethod
    def register_initialization(cls, protocol_instance: AxiomProtocol) -> None:
        """Called strictly by UPR upon successful initialization."""
        cls._INITIALIZED_REGISTRY.add(protocol_instance)

    @classmethod
    def register_shutdown(cls, protocol_instance: AxiomProtocol) -> None:
        """Called strictly by UPR upon shutdown."""
        cls._INITIALIZED_REGISTRY.discard(protocol_instance)

    @classmethod
    def is_initialized(cls, protocol_instance: AxiomProtocol) -> bool:
        """Independent initialization check managed by UPR runtime."""
        return protocol_instance in cls._INITIALIZED_REGISTRY

    @classmethod
    def safe_restore(
        cls, protocol_instance: AxiomProtocol, snapshot: ProtocolSnapshot
    ) -> None:
        """Enforces guard order and executes atomic restore with rollback."""

        if type(snapshot) is not ProtocolSnapshot:
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.RESTORE_TRANSACTION_FAILED,
                    details=FrozenDict({"reason": "Invalid snapshot object type"}),
                )
            )

        if not cls.is_initialized(protocol_instance):
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.NOT_INITIALIZED,
                    details=FrozenDict({"protocol": protocol_instance.meta.name}),
                )
            )

        if snapshot.protocol != protocol_instance.meta.name:
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.PROTOCOL_NAME_MISMATCH,
                    details=FrozenDict(
                        {
                            "expected": protocol_instance.meta.name,
                            "received": snapshot.protocol,
                        }
                    ),
                )
            )

        if not protocol_instance.meta.version.is_runtime_compatible_with(snapshot.version):
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.VERSION_INCOMPATIBLE,
                    details=FrozenDict(
                        {
                            "runtime_version": str(protocol_instance.meta.version),
                            "snapshot_version": str(snapshot.version),
                        }
                    ),
                )
            )

        if not snapshot.verify_integrity():
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.HASH_VERIFICATION_FAILED,
                    details=FrozenDict({"digest": snapshot.digest}),
                )
            )

        try:
            original_snapshot = protocol_instance.snapshot()
        except Exception as capture_err:
            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.SNAPSHOT_CAPTURE_FAILED,
                    details=FrozenDict({"exception_type": type(capture_err).__name__}),
                )
            )

        try:
            protocol_instance.restore(snapshot)
        except Exception as restore_err:
            try:
                protocol_instance.restore(original_snapshot)
            except Exception as rollback_err:
                raise AxiomProtocolError(
                    AxiomErrorPayload(
                        code=ProtocolErrorCode.RESTORE_TRANSACTION_FAILED,
                        details=FrozenDict(
                            {
                                "fatal": "Rollback failed, protocol state corrupted",
                                "restore_error_type": type(restore_err).__name__,
                                "rollback_error_type": type(rollback_err).__name__,
                            }
                        ),
                    )
                )

            raise AxiomProtocolError(
                AxiomErrorPayload(
                    code=ProtocolErrorCode.STATE_TRANSITION_FAILED,
                    details=FrozenDict(
                        {
                            "reason": "Restoration thrown exception, state rolled back successfully",
                            "error_type": type(restore_err).__name__,
                        }
                    ),
                )
            )

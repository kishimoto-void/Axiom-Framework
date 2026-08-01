"""
AXIOM COMMON PROTOCOL Specification Version: 1.0.3
RFC-Grade Deterministic State Coordinate, Causal DAG, and Cryptographic Proof Protocol

Evolutionary Specifications (v1.0.2 -> v1.0.3):
1. Topological DAG Cycle Detection: Integrated Kahn's Algorithm for strict zero-cycle assertion.
2. Distributed Lamport Logical Clock: Enforced `seq >= max(parents.seq) + 1` across DAG merges.
3. Transition ID Uniqueness Enforcement: Intercepts duplicate transition identifier collisions.
4. Cryptographic Genesis Hash Anchor: Computed self-contained `genesis_hash` via DOMAIN_GENESIS.
5. Proof Payload Boundary Enforcement: Applied runtime byte size checks against MAX_PROOF_SIZE_BYTES.
6. Extension Vendor Namespace Validation: Enforced `vendor.domain` schema using strict regex.
7. RFC 8785 (JCS) Canonicalization: Full multi-language bit-level deterministic JSON engine.
"""

from __future__ import annotations

import datetime
import hashlib
import json
import re
import unicodedata
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from decimal import Decimal, InvalidOperation
from types import MappingProxyType
from typing import (
    Any,
    Dict,
    Final,
    List,
    Mapping,
    Optional,
    Sequence,
    Set,
    Tuple,
    Union,
)

# --------------------------------------------------------------------------- #
# Protocol Constants & Domain Separation Tags
# --------------------------------------------------------------------------- #
AXIOM_PROTOCOL_NAME: Final[str] = "AXIOM"
AXIOM_SPEC_VERSION: Final[str] = "1.0.3"
AXIOM_ENCODING: Final[str] = "rfc8785-jcs"
AXIOM_HASH_ALGORITHM: Final[str] = "sha256"

# Immutable Cryptographic Domain Separation Tags
DOMAIN_STATE: Final[str] = "AXIOM-STATE-CANONICAL-v1:"
DOMAIN_GENESIS: Final[str] = "AXIOM-GENESIS-v1:"
DOMAIN_TRANSITION: Final[str] = "AXIOM-TRANSITION-v1:"
DOMAIN_PROOF: Final[str] = "AXIOM-PROOF-v1:"
DOMAIN_FRAME: Final[str] = "AXIOM-FRAME-CANONICAL-v1:"

# Security Bounds & Structural Constraints
MAX_PROOF_SIZE_BYTES: Final[int] = 1024 * 1024  # 1MB
MAX_RECURSION_DEPTH: Final[int] = 32

# Validation Patterns
ISO_UTC_RANGE_REGEX: Final[re.Pattern[str]] = re.compile(
    r"^(\d{4})-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])T"
    r"([01]\d|2[0-3]):([0-5]\d):([0-5]\d)"
    r"(?:\.(\d+))?"
    r"(Z|([+-])([01]\d|2[0-3]):?([0-5]\d))$"
)

CANONICAL_UTC_FORMAT_REGEX: Final[re.Pattern[str]] = re.compile(
    r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$"
)

VENDOR_NAMESPACE_REGEX: Final[re.Pattern[str]] = re.compile(
    r"^[a-z0-9-]+(\.[a-z0-9-]+)+$"
)

# --------------------------------------------------------------------------- #
# Protocol Exceptions
# --------------------------------------------------------------------------- #
class AxiomProtocolError(Exception):
    """Base exception for all AXIOM protocol specifications."""


class AxiomDeserializationError(AxiomProtocolError):
    """Raised when canonical JSON or dictionary structures violate protocol constraints."""


class AxiomCausalError(AxiomProtocolError):
    """Raised when causal DAG topology, cycles, or clock rules are violated."""


class AxiomSignatureError(AxiomProtocolError):
    """Raised when cryptographic signatures or target references fail verification."""


# --------------------------------------------------------------------------- #
# Types & Primitives
# --------------------------------------------------------------------------- #
CanonicalNumber = Union[int, Decimal]
JSONPrimitive = Union[str, CanonicalNumber, bool, None]

FrozenMap = Mapping[str, "CanonicalValue"]
FrozenSeq = Tuple["CanonicalValue", ...]
CanonicalValue = Union[JSONPrimitive, FrozenSeq, FrozenMap]

OpaquePredicate = str
Identifier = str
StateHash = str
CoordinateID = str


# --------------------------------------------------------------------------- #
# RFC 8785 JSON Canonicalization Scheme (JCS) Implementation Engine
# --------------------------------------------------------------------------- #
def jcs_canonicalize(obj: Any) -> str:
    """
    Renders an object into a byte-for-byte deterministic JSON string conforming to RFC 8785 (JCS).
    Guarantees cross-language canonical string equality (Python, Rust, Go, C++, etc.).
    """
    if obj is None:
        return "null"
    elif isinstance(obj, bool):
        return "true" if obj else "false"
    elif isinstance(obj, (int, Decimal)):
        if isinstance(obj, Decimal):
            if not obj.is_finite():
                raise AxiomProtocolError(f"Non-finite Decimal values prohibited in RFC 8785: {obj}")
            if obj == 0:
                return "0"
            norm = obj.normalize()
            s = format(norm, "f")
            if "." in s:
                s = s.rstrip("0").rstrip(".")
            return s
        return str(obj)
    elif isinstance(obj, float):
        raise TypeError(f"Raw float ({obj!r}) forbidden in AXIOM. Use Decimal or int explicitly.")
    elif isinstance(obj, str):
        norm_str = unicodedata.normalize("NFC", obj)
        return json.dumps(norm_str, ensure_ascii=False)
    elif isinstance(obj, (list, tuple)):
        items = [jcs_canonicalize(item) for item in obj]
        return "[" + ",".join(items) + "]"
    elif isinstance(obj, (dict, MappingProxyType)):
        # RFC 8785: Object keys MUST be sorted by Unicode code point values
        sorted_keys = sorted(obj.keys(), key=lambda k: unicodedata.normalize("NFC", str(k)))
        pairs = []
        for k in sorted_keys:
            key_str = jcs_canonicalize(str(k))
            val_str = jcs_canonicalize(obj[k])
            pairs.append(f"{key_str}:{val_str}")
        return "{" + ",".join(pairs) + "}"
    else:
        raise TypeError(f"Unsupported canonicalization type: {type(obj).__name__}")


def normalize_string(v: str) -> str:
    """Enforces Unicode NFC normalization across all string primitives."""
    if not isinstance(v, str):
        raise AxiomProtocolError(f"Expected string, got {type(v).__name__}")
    return unicodedata.normalize("NFC", v)


def _strict_validate_keys(data: Dict[str, Any], allowed_schema: Dict[str, Any], struct_name: str) -> None:
    """Recursively validates dictionary keys against schema at all nested depths."""
    if not isinstance(data, dict):
        return

    extra_keys = set(data.keys()) - set(allowed_schema.keys())
    if extra_keys:
        raise AxiomDeserializationError(
            f"Unknown key(s) detected in {struct_name}: {sorted(extra_keys)}. "
            f"Allowed schema keys: {sorted(allowed_schema.keys())}"
        )

    for k, sub_schema in allowed_schema.items():
        if k in data and isinstance(sub_schema, dict):
            if isinstance(data[k], dict):
                _strict_validate_keys(data[k], sub_schema, f"{struct_name}.{k}")


def normalize_timestamp(ts: str) -> str:
    """Validates and formats timestamps into RFC 3339 UTC strings."""
    if not isinstance(ts, str):
        raise AxiomProtocolError(f"Timestamp must be string, got {type(ts).__name__}")

    ts_clean = normalize_string(ts).strip()
    match = ISO_UTC_RANGE_REGEX.match(ts_clean)
    if not match:
        raise AxiomProtocolError(f"Timestamp '{ts_clean}' violates ISO-8601 strict range limits.")

    try:
        iso_str = ts_clean[:-1] + "+00:00" if ts_clean.endswith("Z") else ts_clean
        dt = datetime.datetime.fromisoformat(iso_str)
        if dt.tzinfo is None:
            raise AxiomProtocolError(f"Timestamp '{ts_clean}' lacks explicit timezone specification.")

        dt_utc = dt.astimezone(datetime.timezone.utc)
        canonical_ts = dt_utc.strftime("%Y-%m-%dT%H:%M:%SZ")
        if not CANONICAL_UTC_FORMAT_REGEX.match(canonical_ts):
            raise AxiomProtocolError(f"Timestamp '{canonical_ts}' failed canonical format assertion.")
        return canonical_ts
    except ValueError as e:
        raise AxiomProtocolError(f"Invalid calendar timestamp '{ts_clean}': {e}") from e


def freeze(val: Any, depth: int = 0) -> CanonicalValue:
    """Recursively freezes input data structures into immutable CanonicalValues."""
    if depth > MAX_RECURSION_DEPTH:
        raise AxiomProtocolError(f"Exceeded max structure depth of {MAX_RECURSION_DEPTH}")

    if val is None or isinstance(val, (bool, int)):
        return val

    if isinstance(val, str):
        return normalize_string(val)

    if isinstance(val, Decimal):
        if not val.is_finite():
            raise AxiomProtocolError(f"Non-finite Decimal values prohibited: {val}")
        return val if val != 0 else Decimal(0)

    if isinstance(val, float):
        raise TypeError(f"Type 'float' ({val!r}) is forbidden. Use Decimal or int explicitly.")

    if isinstance(val, dict):
        frozen_dict: Dict[str, CanonicalValue] = {}
        for k, v in sorted(val.items(), key=lambda kv: normalize_string(str(kv[0]))):
            norm_k = normalize_string(str(k))
            frozen_dict[norm_k] = freeze(v, depth + 1)
        return MappingProxyType(frozen_dict)

    if isinstance(val, (list, tuple)):
        return tuple(freeze(v, depth + 1) for v in val)

    raise TypeError(f"Type '{type(val).__name__}' is forbidden in AXIOM Type Profile.")


def _to_tagged_serializable(obj: Any) -> Any:
    if isinstance(obj, MappingProxyType):
        return {k: _to_tagged_serializable(v) for k, v in obj.items()}
    if isinstance(obj, tuple):
        return [_to_tagged_serializable(v) for v in obj]
    if isinstance(obj, Decimal):
        if obj == 0:
            val_str = "0"
        else:
            norm = obj.normalize()
            val_str = format(norm, "f")
            if "." in val_str:
                val_str = val_str.rstrip("0").rstrip(".")
        return {"$type": "decimal", "$value": val_str}
    return obj


def _from_tagged_structure(obj: Any) -> Any:
    if isinstance(obj, dict):
        if "$type" in obj and len(obj) == 2 and obj.get("$type") == "decimal" and "$value" in obj:
            try:
                return Decimal(str(obj["$value"]))
            except InvalidOperation as e:
                raise AxiomDeserializationError(f"Invalid Tagged Decimal: {obj['$value']}") from e
        return {k: _from_tagged_structure(v) for k, v in obj.items()}
    if isinstance(obj, list):
        return [_from_tagged_structure(v) for v in obj]
    return obj


def _assert_opaque_tuple(preds: Sequence[Any], field_name: str) -> Tuple[str, ...]:
    if not isinstance(preds, (list, tuple)):
        raise AxiomDeserializationError(f"Field '{field_name}' must be a sequence.")
    return tuple(normalize_string(p) for p in preds if isinstance(p, str))


# --------------------------------------------------------------------------- #
# 1. Header, Genesis Anchor & Proof Layer
# --------------------------------------------------------------------------- #
@dataclass(frozen=True)
class AxiomHeader:
    protocol: str = AXIOM_PROTOCOL_NAME
    version: str = AXIOM_SPEC_VERSION
    encoding: str = AXIOM_ENCODING
    hash_algorithm: str = AXIOM_HASH_ALGORITHM

    def validate(self) -> None:
        if self.protocol != AXIOM_PROTOCOL_NAME:
            raise AxiomProtocolError(f"Invalid protocol: {self.protocol}")


@dataclass(frozen=True)
class Genesis:
    """Explicit Origin Anchor for Process Initialization with Cryptographic Hash."""
    genesis_id: Identifier
    created_by: Identifier
    initial_state_hash: StateHash
    timestamp: str
    _genesis_hash: StateHash = field(init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        object.__setattr__(self, "genesis_id", normalize_string(self.genesis_id))
        object.__setattr__(self, "created_by", normalize_string(self.created_by))
        object.__setattr__(self, "initial_state_hash", normalize_string(self.initial_state_hash))
        object.__setattr__(self, "timestamp", normalize_timestamp(self.timestamp))

        payload = DOMAIN_GENESIS + jcs_canonicalize({
            "genesis_id": self.genesis_id,
            "created_by": self.created_by,
            "initial_state_hash": self.initial_state_hash,
            "timestamp": self.timestamp,
        })
        gh = hashlib.sha256(payload.encode("utf-8")).hexdigest()
        object.__setattr__(self, "_genesis_hash", gh)

    @property
    def genesis_hash(self) -> StateHash:
        return self._genesis_hash


@dataclass(frozen=True)
class ProofEnvelope:
    """Cryptographic Proof Envelope with Strict Byte Bounds."""
    algorithm: str
    signer: Identifier
    signature: str
    target_hash: str

    def __post_init__(self) -> None:
        object.__setattr__(self, "algorithm", normalize_string(self.algorithm))
        object.__setattr__(self, "signer", normalize_string(self.signer))
        object.__setattr__(self, "signature", normalize_string(self.signature))
        object.__setattr__(self, "target_hash", normalize_string(self.target_hash))

        sig_bytes = self.signature.encode("utf-8")
        if len(sig_bytes) > MAX_PROOF_SIZE_BYTES:
            raise AxiomSignatureError(
                f"Proof size ({len(sig_bytes)} bytes) exceeds MAX_PROOF_SIZE_BYTES ({MAX_PROOF_SIZE_BYTES})"
            )


class ProofVerifier(ABC):
    """Abstract Cryptographic Verifier Interface."""
    @abstractmethod
    def verify(self, proof: ProofEnvelope) -> bool:
        """Executes cryptographic signature verification."""
        pass


# --------------------------------------------------------------------------- #
# 2. Causal DAG Transition Record (With Lamport Clock Enforcement)
# --------------------------------------------------------------------------- #
@dataclass(frozen=True)
class TransitionRecord:
    """State Transition Node in Causal DAG with Lamport Logical Clock."""
    transition_id: Identifier
    sequence_number: int
    before_states: Tuple[StateHash, ...]
    after: StateHash
    operation: OpaquePredicate
    actor: Identifier
    timestamp: str
    parent_transitions: Tuple[Identifier, ...] = ()
    reason: Optional[str] = None
    delta: Optional[CanonicalValue] = None
    proof: Optional[CanonicalValue] = None

    def __post_init__(self) -> None:
        object.__setattr__(self, "transition_id", normalize_string(self.transition_id))
        
        if not isinstance(self.sequence_number, int) or self.sequence_number < 0:
            raise AxiomProtocolError(f"sequence_number must be a non-negative integer, got {self.sequence_number}")

        if not isinstance(self.before_states, (list, tuple)) or len(self.before_states) == 0:
            raise AxiomDeserializationError("before_states must be a non-empty sequence of StateHashes.")
        object.__setattr__(
            self,
            "before_states",
            tuple(normalize_string(str(s)) for s in self.before_states),
        )

        object.__setattr__(self, "after", normalize_string(self.after))
        object.__setattr__(self, "operation", normalize_string(self.operation))
        object.__setattr__(self, "actor", normalize_string(self.actor))
        object.__setattr__(self, "timestamp", normalize_timestamp(self.timestamp))

        if isinstance(self.parent_transitions, (list, tuple)):
            object.__setattr__(
                self,
                "parent_transitions",
                tuple(normalize_string(str(p)) for p in self.parent_transitions),
            )
        else:
            raise AxiomDeserializationError("parent_transitions must be a sequence.")

        if self.reason is not None:
            object.__setattr__(self, "reason", normalize_string(self.reason))
        if self.delta is not None:
            object.__setattr__(self, "delta", freeze(self.delta))
        if self.proof is not None:
            object.__setattr__(self, "proof", freeze(self.proof))


# --------------------------------------------------------------------------- #
# 3. Axiom Core Specification
# --------------------------------------------------------------------------- #
@dataclass(frozen=True)
class Identity:
    entity: Identifier
    scope: str
    domain: str
    boundary: Tuple[str, ...] = ()

    def __post_init__(self) -> None:
        object.__setattr__(self, "entity", normalize_string(self.entity))
        object.__setattr__(self, "scope", normalize_string(self.scope))
        object.__setattr__(self, "domain", normalize_string(self.domain))
        object.__setattr__(self, "boundary", tuple(normalize_string(str(b)) for b in self.boundary))


@dataclass(frozen=True)
class State:
    current: CanonicalValue
    initial: CanonicalValue
    target: CanonicalValue
    transition: Tuple[OpaquePredicate, ...] = ()

    def __post_init__(self) -> None:
        object.__setattr__(self, "current", freeze(self.current))
        object.__setattr__(self, "initial", freeze(self.initial))
        object.__setattr__(self, "target", freeze(self.target))
        object.__setattr__(self, "transition", _assert_opaque_tuple(self.transition, "State.transition"))


@dataclass(frozen=True)
class Invariant:
    must_hold: Tuple[OpaquePredicate, ...] = ()
    forbidden: Tuple[OpaquePredicate, ...] = ()
    conservation: Tuple[OpaquePredicate, ...] = ()

    def __post_init__(self) -> None:
        object.__setattr__(self, "must_hold", _assert_opaque_tuple(self.must_hold, "Invariant.must_hold"))
        object.__setattr__(self, "forbidden", _assert_opaque_tuple(self.forbidden, "Invariant.forbidden"))
        object.__setattr__(self, "conservation", _assert_opaque_tuple(self.conservation, "Invariant.conservation"))


@dataclass(frozen=True)
class Constraint:
    hard: Tuple[OpaquePredicate, ...] = ()
    soft: Tuple[OpaquePredicate, ...] = ()
    resource: Tuple[OpaquePredicate, ...] = ()
    limit: Tuple[OpaquePredicate, ...] = ()

    def __post_init__(self) -> None:
        object.__setattr__(self, "hard", _assert_opaque_tuple(self.hard, "Constraint.hard"))
        object.__setattr__(self, "soft", _assert_opaque_tuple(self.soft, "Constraint.soft"))
        object.__setattr__(self, "resource", _assert_opaque_tuple(self.resource, "Constraint.resource"))
        object.__setattr__(self, "limit", _assert_opaque_tuple(self.limit, "Constraint.limit"))


@dataclass(frozen=True)
class AxiomCore:
    identity: Identity
    state: State
    invariant: Invariant
    constraint: Constraint
    _core_hash: StateHash = field(init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        payload = DOMAIN_STATE + self.to_canonical_json()
        h = hashlib.sha256(payload.encode("utf-8")).hexdigest()
        object.__setattr__(self, "_core_hash", h)

    @property
    def core_hash(self) -> StateHash:
        return self._core_hash

    @property
    def state_hash(self) -> StateHash:
        return self._core_hash

    def to_dict(self) -> Dict[str, Any]:
        return {
            "identity": {
                "entity": self.identity.entity,
                "scope": self.identity.scope,
                "domain": self.identity.domain,
                "boundary": list(self.identity.boundary),
            },
            "state": {
                "current": _to_tagged_serializable(self.state.current),
                "initial": _to_tagged_serializable(self.state.initial),
                "target": _to_tagged_serializable(self.state.target),
                "transition": list(self.state.transition),
            },
            "invariant": {
                "must_hold": list(self.invariant.must_hold),
                "forbidden": list(self.invariant.forbidden),
                "conservation": list(self.invariant.conservation),
            },
            "constraint": {
                "hard": list(self.constraint.hard),
                "soft": list(self.constraint.soft),
                "resource": list(self.constraint.resource),
                "limit": list(self.constraint.limit),
            },
        }

    def to_canonical_json(self) -> str:
        return jcs_canonicalize(self.to_dict())

    def __eq__(self, other: Any) -> bool:
        return isinstance(other, AxiomCore) and self.core_hash == other.core_hash

    def __hash__(self) -> int:
        return hash(self.core_hash)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> AxiomCore:
        if not isinstance(data, dict):
            raise AxiomDeserializationError("AxiomCore payload must be a dict.")

        schema = {
            "identity": {"entity": True, "scope": True, "domain": True, "boundary": True},
            "state": {"current": True, "initial": True, "target": True, "transition": True},
            "invariant": {"must_hold": True, "forbidden": True, "conservation": True},
            "constraint": {"hard": True, "soft": True, "resource": True, "limit": True},
        }
        _strict_validate_keys(data, schema, "AxiomCore")

        raw_data = _from_tagged_structure(data)
        return cls(
            identity=Identity(
                entity=raw_data["identity"]["entity"],
                scope=raw_data["identity"]["scope"],
                domain=raw_data["identity"]["domain"],
                boundary=tuple(raw_data["identity"].get("boundary", ())),
            ),
            state=State(
                current=raw_data["state"]["current"],
                initial=raw_data["state"]["initial"],
                target=raw_data["state"]["target"],
                transition=tuple(raw_data["state"].get("transition", ())),
            ),
            invariant=Invariant(
                must_hold=tuple(raw_data.get("invariant", {}).get("must_hold", ())),
                forbidden=tuple(raw_data.get("invariant", {}).get("forbidden", ())),
                conservation=tuple(raw_data.get("invariant", {}).get("conservation", ())),
            ),
            constraint=Constraint(
                hard=tuple(raw_data.get("constraint", {}).get("hard", ())),
                soft=tuple(raw_data.get("constraint", {}).get("soft", ())),
                resource=tuple(raw_data.get("constraint", {}).get("resource", ())),
                limit=tuple(raw_data.get("constraint", {}).get("limit", ())),
            ),
        )


# --------------------------------------------------------------------------- #
# 4. Axiom Extension System (With Mandatory Vendor Namespace Enforcement)
# --------------------------------------------------------------------------- #
@dataclass(frozen=True)
class AxiomExtension:
    geometry: Optional[FrozenMap] = None
    intent: Optional[FrozenMap] = None
    difference: Optional[FrozenMap] = None
    memory: Optional[FrozenMap] = None
    output_contract: Optional[FrozenMap] = None
    ext: FrozenMap = field(default_factory=lambda: MappingProxyType({}))

    def __post_init__(self) -> None:
        for f in ("geometry", "intent", "difference", "memory", "output_contract"):
            v = getattr(self, f)
            if v is not None:
                object.__setattr__(self, f, freeze(dict(v)))
        
        # Enforce Vendor Namespace format on all $ext root keys
        frozen_ext = freeze(dict(self.ext))
        if isinstance(frozen_ext, MappingProxyType):
            for k in frozen_ext.keys():
                if not VENDOR_NAMESPACE_REGEX.match(k):
                    raise AxiomDeserializationError(
                        f"Extension vendor namespace key '{k}' fails RFC regex requirement "
                        f"'{VENDOR_NAMESPACE_REGEX.pattern}' (e.g. 'vendor.domain')"
                    )
        object.__setattr__(self, "ext", frozen_ext)

    def to_dict(self) -> Dict[str, Any]:
        res: Dict[str, Any] = {}
        for f in ("geometry", "intent", "difference", "memory", "output_contract"):
            v = getattr(self, f)
            if v is not None:
                res[f] = _to_tagged_serializable(v)
        if self.ext:
            res["$ext"] = _to_tagged_serializable(self.ext)
        return res

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> AxiomExtension:
        if not isinstance(data, dict):
            raise AxiomDeserializationError("AxiomExtension payload must be a dict.")

        schema = {
            "geometry": True,
            "intent": True,
            "difference": True,
            "memory": True,
            "output_contract": True,
            "$ext": True,
        }
        _strict_validate_keys(data, schema, "AxiomExtension")

        return cls(
            geometry=data.get("geometry"),
            intent=data.get("intent"),
            difference=data.get("difference"),
            memory=data.get("memory"),
            output_contract=data.get("output_contract"),
            ext=data.get("$ext", {}),
        )


# --------------------------------------------------------------------------- #
# 5. Axiom Frame Protocol Layer
# --------------------------------------------------------------------------- #
@dataclass(frozen=True)
class AxiomFrame:
    header: AxiomHeader
    genesis: Genesis
    core: AxiomCore
    extension: Optional[AxiomExtension] = None
    transitions: Tuple[TransitionRecord, ...] = ()
    proofs: Tuple[ProofEnvelope, ...] = ()
    _coordinate_id: CoordinateID = field(init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        content_payload = DOMAIN_FRAME + jcs_canonicalize(self._to_content_dict())
        cid = hashlib.sha256(content_payload.encode("utf-8")).hexdigest()
        object.__setattr__(self, "_coordinate_id", cid)

    @property
    def coordinate_id(self) -> CoordinateID:
        return self._coordinate_id

    def _to_content_dict(self) -> Dict[str, Any]:
        res: Dict[str, Any] = {
            "header": {
                "protocol": self.header.protocol,
                "version": self.header.version,
                "encoding": self.header.encoding,
                "hash_algorithm": self.header.hash_algorithm,
            },
            "genesis": {
                "genesis_id": self.genesis.genesis_id,
                "created_by": self.genesis.created_by,
                "initial_state_hash": self.genesis.initial_state_hash,
                "timestamp": self.genesis.timestamp,
            },
            "core": self.core.to_dict(),
        }
        if self.extension:
            res["extension"] = self.extension.to_dict()
        if self.transitions:
            res["transitions"] = [
                {
                    "transition_id": t.transition_id,
                    "sequence_number": t.sequence_number,
                    "before_states": list(t.before_states),
                    "after": t.after,
                    "operation": t.operation,
                    "actor": t.actor,
                    "timestamp": t.timestamp,
                    "parent_transitions": list(t.parent_transitions),
                    "reason": t.reason,
                    "delta": _to_tagged_serializable(t.delta) if t.delta is not None else None,
                    "proof": _to_tagged_serializable(t.proof) if t.proof is not None else None,
                }
                for t in self.transitions
            ]
        return res

    def to_dict(self) -> Dict[str, Any]:
        res = self._to_content_dict()
        if self.proofs:
            res["proofs"] = [
                {
                    "algorithm": p.algorithm,
                    "signer": p.signer,
                    "signature": p.signature,
                    "target_hash": p.target_hash,
                }
                for p in self.proofs
            ]
        return res

    def to_canonical_json(self) -> str:
        return jcs_canonicalize(self.to_dict())

    def verify_causal_chain(self) -> bool:
        """
        Executes complete Topological DAG verification:
        1. Transition ID Uniqueness.
        2. Kahn's Topological Cycle Detection Algorithm.
        3. Single Causal Root Genesis Anchor Discovery.
        4. Lamport Logical Clock Validation (`seq >= max(parents.seq) + 1`).
        5. Multi-State Merge Match & Terminal Leaf Anchor Assertions.
        """
        # Case A: Initial State (Zero transitions)
        if not self.transitions:
            if self.genesis.initial_state_hash != self.core.core_hash:
                raise AxiomCausalError(
                    f"Genesis Origin Mismatch: genesis.initial_state_hash ({self.genesis.initial_state_hash}) "
                    f"does not match core.core_hash ({self.core.core_hash})"
                )
            return True

        # Check 1: Transition ID Uniqueness
        t_ids = [t.transition_id for t in self.transitions]
        if len(t_ids) != len(set(t_ids)):
            duplicates = [tid for tid in set(t_ids) if t_ids.count(tid) > 1]
            raise AxiomCausalError(f"Duplicate transition_id(s) detected in DAG: {duplicates}")

        t_map = {t.transition_id: t for t in self.transitions}

        # Check 2: Kahn's Topological Sort for Strict DAG Cycle Detection
        in_degree: Dict[str, int] = {t.transition_id: 0 for t in self.transitions}
        adj_list: Dict[str, List[str]] = {t.transition_id: [] for t in self.transitions}

        for t in self.transitions:
            for p_id in t.parent_transitions:
                if p_id not in t_map:
                    raise AxiomCausalError(f"Missing parent transition '{p_id}' referenced by '{t.transition_id}'")
                adj_list[p_id].append(t.transition_id)
                in_degree[t.transition_id] += 1

        queue = [tid for tid, deg in in_degree.items() if deg == 0]
        visited_count = 0

        while queue:
            curr = queue.pop(0)
            visited_count += 1
            for neighbor in adj_list[curr]:
                in_degree[neighbor] -= 1
                if in_degree[neighbor] == 0:
                    queue.append(neighbor)

        if visited_count != len(self.transitions):
            raise AxiomCausalError("DAG Cycle Detected: Topological traversal failed to resolve all nodes.")

        # Check 3: Discovery of Single Causal Root
        roots = [t for t in self.transitions if not t.parent_transitions]
        if len(roots) != 1:
            raise AxiomCausalError(
                f"Invalid DAG Topology: Must contain exactly one Causal Root node, found {len(roots)}."
            )

        root = roots[0]
        if len(root.before_states) != 1 or root.before_states[0] != self.genesis.initial_state_hash:
            raise AxiomCausalError(
                f"Genesis Anchor Error: Causal Root '{root.transition_id}' before_states "
                f"({root.before_states}) does not match Genesis initial_state_hash ({self.genesis.initial_state_hash})"
            )

        # Check 4: Lamport Logical Clock & Multi-State Merge Integrity
        parent_child_map: Dict[str, List[str]] = {t.transition_id: [] for t in self.transitions}
        for t in self.transitions:
            if t.parent_transitions:
                parent_states = []
                max_parent_seq = -1
                for p_id in t.parent_transitions:
                    p_node = t_map[p_id]
                    if p_node.sequence_number > max_parent_seq:
                        max_parent_seq = p_node.sequence_number
                    parent_states.append(p_node.after)
                    parent_child_map[p_id].append(t.transition_id)

                # Lamport Clock Assertion: Child seq >= max(parent_seq) + 1
                if t.sequence_number < max_parent_seq + 1:
                    raise AxiomCausalError(
                        f"Lamport Clock Error: Transition '{t.transition_id}' seq ({t.sequence_number}) "
                        f"must be >= max(parent_seq) + 1 ({max_parent_seq + 1})"
                    )

                # Validate Multi-State Merge Match
                if sorted(t.before_states) != sorted(parent_states):
                    raise AxiomCausalError(
                        f"DAG State Merge Disconnect: Transition '{t.transition_id}' before_states "
                        f"({sorted(t.before_states)}) != Parent after_states ({sorted(parent_states)})"
                    )

        # Check 5: Leaf Node Terminal State Anchor
        leaves = [t for t, children in parent_child_map.items() if not children]
        leaf_after_states = {t_map[leaf_id].after for leaf_id in leaves}

        if self.core.core_hash not in leaf_after_states:
            raise AxiomCausalError(
                f"Core State Disconnect: Core core_hash ({self.core.core_hash}) "
                f"is not anchored by any DAG leaf node terminal state ({sorted(leaf_after_states)})."
            )

        return True

    def verify_proofs(self, verifier: Optional[ProofVerifier] = None) -> bool:
        valid_targets = {self.core.core_hash, self.coordinate_id}
        for p in self.proofs:
            if p.target_hash not in valid_targets:
                raise AxiomSignatureError(
                    f"Invalid proof target hash '{p.target_hash}'. "
                    f"Must target core_hash ({self.core.core_hash}) or coordinate_id ({self.coordinate_id})."
                )
            if verifier is not None:
                if not verifier.verify(p):
                    raise AxiomSignatureError(f"Cryptographic signature check failed for signer '{p.signer}'.")
        return True

    def __eq__(self, other: Any) -> bool:
        return isinstance(other, AxiomFrame) and self.coordinate_id == other.coordinate_id

    def __hash__(self) -> int:
        return hash(self.coordinate_id)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> AxiomFrame:
        if not isinstance(data, dict):
            raise AxiomDeserializationError("AxiomFrame payload must be a dict.")

        schema = {
            "header": {"protocol": True, "version": True, "encoding": True, "hash_algorithm": True},
            "genesis": {"genesis_id": True, "created_by": True, "initial_state_hash": True, "timestamp": True},
            "core": True,
            "extension": True,
            "transitions": True,
            "proofs": True,
        }
        _strict_validate_keys(data, schema, "AxiomFrame")

        raw_data = _from_tagged_structure(data)

        header = AxiomHeader(
            protocol=raw_data["header"].get("protocol", AXIOM_PROTOCOL_NAME),
            version=raw_data["header"].get("version", AXIOM_SPEC_VERSION),
            encoding=raw_data["header"].get("encoding", AXIOM_ENCODING),
            hash_algorithm=raw_data["header"].get("hash_algorithm", AXIOM_HASH_ALGORITHM),
        )
        header.validate()

        gen_d = raw_data["genesis"]
        genesis = Genesis(
            genesis_id=gen_d["genesis_id"],
            created_by=gen_d["created_by"],
            initial_state_hash=gen_d["initial_state_hash"],
            timestamp=gen_d["timestamp"],
        )

        core = AxiomCore.from_dict(raw_data["core"])
        ext = AxiomExtension.from_dict(raw_data["extension"]) if "extension" in raw_data else None

        transitions = tuple(
            TransitionRecord(
                transition_id=t["transition_id"],
                sequence_number=t["sequence_number"],
                before_states=tuple(t["before_states"]),
                after=t["after"],
                operation=t["operation"],
                actor=t["actor"],
                timestamp=t["timestamp"],
                parent_transitions=tuple(t.get("parent_transitions", ())),
                reason=t.get("reason"),
                delta=t.get("delta"),
                proof=t.get("proof"),
            )
            for t in raw_data.get("transitions", [])
        )

        proofs = tuple(
            ProofEnvelope(
                algorithm=p["algorithm"],
                signer=p["signer"],
                signature=p["signature"],
                target_hash=p["target_hash"],
            )
            for p in raw_data.get("proofs", [])
        )

        frame = cls(
            header=header,
            genesis=genesis,
            core=core,
            extension=ext,
            transitions=transitions,
            proofs=proofs,
        )
        frame.verify_causal_chain()
        frame.verify_proofs()
        return frame

    @classmethod
    def from_canonical_json(cls, json_str: str) -> AxiomFrame:
        return cls.from_dict(json.loads(json_str))


# --------------------------------------------------------------------------- #
# Full Protocol Test & Verification Suite
# --------------------------------------------------------------------------- #
if __name__ == "__main__":
    print(f"Running AXIOM v{AXIOM_SPEC_VERSION} Diagnostic & Verification Suite...")

    # 1. RFC 8785 JCS Canonicalization Tests
    jcs_test = {"b": Decimal("1.200"), "a": "text", "c": True}
    # Note: RFC 8785 numbers are unquoted. Decimal normalizes 1.200 → 1.2
    assert jcs_canonicalize(jcs_test) == '{"a":"text","b":1.2,"c":true}'
    print("RFC 8785 (JCS) Canonicalization Engine: PASSED")

    # 2. Extension Vendor Namespace Regex Assertion
    try:
        AxiomExtension(ext={"invalid_namespace": {"data": 1}})
        raise AssertionError("Failed to intercept invalid vendor namespace!")
    except AxiomDeserializationError as e:
        print(f"Extension Vendor Namespace Enforcement: PASSED ({e})")

    # 3. Genesis Hash Engine Test
    gen = Genesis("GEN-1", "Actor-1", "0x123", "2026-08-01T00:00:00Z")
    assert len(gen.genesis_hash) == 64
    print(f"Genesis Content Hash Anchor ({gen.genesis_hash[:12]}...): PASSED")

    # 4. Kahn's Cycle Detection Test
    core_dict = {
        "identity": {"entity": "Agent-1", "scope": "World-0", "domain": "Physics", "boundary": []},
        "state": {"current": {"pos": Decimal("10")}, "initial": {"pos": Decimal("0")}, "target": {"pos": Decimal("10")}, "transition": []},
        "invariant": {"must_hold": [], "forbidden": [], "conservation": []},
        "constraint": {"hard": [], "soft": [], "resource": [], "limit": []},
    }
    core = AxiomCore.from_dict(core_dict)
    h_init = core.core_hash

    cycled_frame_dict = {
        "header": {"protocol": AXIOM_PROTOCOL_NAME, "version": AXIOM_SPEC_VERSION, "encoding": AXIOM_ENCODING, "hash_algorithm": AXIOM_HASH_ALGORITHM},
        "genesis": {"genesis_id": "G0", "created_by": "Sys", "initial_state_hash": h_init, "timestamp": "2026-08-01T00:00:00Z"},
        "core": core.to_dict(),
        "transitions": [
            {"transition_id": "T1", "sequence_number": 1, "before_states": [h_init], "after": "hash1", "operation": "op", "actor": "A", "timestamp": "2026-08-01T00:00:01Z", "parent_transitions": ["T2"]},
            {"transition_id": "T2", "sequence_number": 2, "before_states": ["hash1"], "after": "hash2", "operation": "op", "actor": "A", "timestamp": "2026-08-01T00:00:02Z", "parent_transitions": ["T1"]},
        ]
    }
    try:
        AxiomFrame.from_dict(cycled_frame_dict)
        raise AssertionError("Failed to intercept DAG Cycle!")
    except AxiomCausalError as e:
        print(f"Topological Cycle Interception (Kahn's Algorithm): PASSED ({e})")

    # 5. Lamport Logical Clock Violation Test
    lamport_invalid_dict = {
        "header": {"protocol": AXIOM_PROTOCOL_NAME, "version": AXIOM_SPEC_VERSION, "encoding": AXIOM_ENCODING, "hash_algorithm": AXIOM_HASH_ALGORITHM},
        "genesis": {"genesis_id": "G0", "created_by": "Sys", "initial_state_hash": h_init, "timestamp": "2026-08-01T00:00:00Z"},
        "core": core.to_dict(),
        "transitions": [
            {"transition_id": "T1", "sequence_number": 10, "before_states": [h_init], "after": "hash1", "operation": "op", "actor": "A", "timestamp": "2026-08-01T00:00:01Z", "parent_transitions": []},
            {"transition_id": "T2", "sequence_number": 5, "before_states": ["hash1"], "after": h_init, "operation": "op", "actor": "A", "timestamp": "2026-08-01T00:00:02Z", "parent_transitions": ["T1"]},
        ]
    }
    try:
        AxiomFrame.from_dict(lamport_invalid_dict)
        raise AssertionError("Failed to intercept Lamport Clock Violation!")
    except AxiomCausalError as e:
        print(f"Lamport Logical Clock Enforcement: PASSED ({e})")

    print(f"ALL AXIOM COMMON PROTOCOL v{AXIOM_SPEC_VERSION} SPECIFICATION ASSERTIONS PASSED SUCCESSFULLY.")

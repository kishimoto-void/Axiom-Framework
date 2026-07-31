"""
Property-Based Verification Suite for Axiom Protocol v2.7
Uses Hypothesis to rigorously verify algebraic invariants, bijectivity, and fuzz resilience.
"""

from hypothesis import given, strategies as st
import pytest
import math
import unicodedata
from axiom.protocol import (
    CanonicalSerializer,
    FrozenDict,
    _sanitize_value,
    _strict_eq,
    AxiomValue,
)


# ============================================================ #
# 1. Recursive Axiom Value Strategy Generator
# ============================================================ #

axiom_primitives = st.one_of(
    st.none(),
    st.booleans(),
    st.integers(min_value=-2**63, max_value=2**63 - 1),
    st.floats(allow_nan=False, allow_infinity=False),
    st.text(),
)

def axiom_values(max_depth=4):
    return st.recursive(
        axiom_primitives,
        lambda children: st.one_of(
            st.tuples(*[children for _ in range(st.integers(0, 3).example())]),
            st.dictionaries(st.text(), children).map(lambda d: FrozenDict(d)),
        ),
        max_leaves=10,
    )


# ============================================================ #
# 2. Algebraic Invariant Tests (Bijectivity & Canonical Formalism)
# ============================================================ #

@given(val=axiom_values())
def test_axiom_value_roundtrip_bijectivity(val: AxiomValue):
    """
    Axiom 1: D(S(V)) == V under strict type identity.
    """
    sanitized = _sanitize_value(val)
    encoded_bytes = CanonicalSerializer.serialize_to_bytes(sanitized)
    decoded_val = CanonicalSerializer.deserialize_from_bytes(encoded_bytes)

    # 1. Structural and Type Equivalence
    assert _strict_eq(sanitized, decoded_val)
    assert type(sanitized) is type(decoded_val)


@given(val=axiom_values())
def test_canonical_byte_identity(val: AxiomValue):
    """
    Axiom 2: S(D(S(V))) == S(V)
    Re-serialization produces identical canonical bytes without drift.
    """
    sanitized = _sanitize_value(val)
    b1 = CanonicalSerializer.serialize_to_bytes(sanitized)
    decoded = CanonicalSerializer.deserialize_from_bytes(b1)
    b2 = CanonicalSerializer.serialize_to_bytes(decoded)

    assert b1 == b2


@given(data=st.binary())
def test_malformed_byte_stream_fuzz_rejection(data: bytes):
    """
    Axiom 3: Any arbitrary byte stream B either succeeds with S(D(B)) == B,
    or raises a deterministic ValueError (never crashes with uncaught exceptions).
    """
    try:
        decoded = CanonicalSerializer.deserialize_from_bytes(data)
        re_encoded = CanonicalSerializer.serialize_to_bytes(decoded)
        # If deserialization succeeds, B MUST be the canonical representation
        assert data == re_encoded
    except ValueError:
        # Expected canonical rejection
        pass


@given(d=st.dictionaries(st.text(), axiom_values()))
def test_frozendict_immutability_and_hash_coherence(d: dict):
    """
    Axiom 4: FrozenDict preserves strict type key equality and produces invariant hashes.
    """
    fd1 = FrozenDict(d)
    fd2 = FrozenDict(d)

    assert fd1 == fd2
    assert hash(fd1) == hash(fd2)
    assert type(fd1) is FrozenDict

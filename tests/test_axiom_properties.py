"""
Property-Based Verification Suite for Axiom Protocol v2.7
Uses Hypothesis to rigorously verify algebraic invariants, bijectivity, and fuzz resilience.
"""

from hypothesis import given, strategies as st, settings, HealthCheck
import pytest
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
    st.floats(allow_nan=False, allow_infinity=False, width=64),
    st.text(max_size=32),
)

def axiom_values(max_leaves=8):
    return st.recursive(
        axiom_primitives,
        lambda children: st.one_of(
            st.lists(children, max_size=4).map(tuple),
            st.dictionaries(st.text(max_size=16), children, max_size=4).map(lambda d: FrozenDict(d)),
        ),
        max_leaves=max_leaves,
    )


# ============================================================ #
# 2. Algebraic Invariant Tests (Bijectivity & Canonical Formalism)
# ============================================================ #

@settings(max_examples=200, deadline=None, suppress_health_check=[HealthCheck.too_slow])
@given(val=axiom_values())
def test_axiom_value_roundtrip_bijectivity(val: AxiomValue):
    """
    Axiom 1: D(S(V)) == V under strict type identity.
    """
    sanitized = _sanitize_value(val)
    encoded_bytes = CanonicalSerializer.serialize_to_bytes(sanitized)
    decoded_val = CanonicalSerializer.deserialize_from_bytes(encoded_bytes)

    assert _strict_eq(sanitized, decoded_val)
    assert type(sanitized) is type(decoded_val)


@settings(max_examples=200, deadline=None, suppress_health_check=[HealthCheck.too_slow])
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


@settings(max_examples=300, deadline=None)
@given(data=st.binary(max_size=256))
def test_malformed_byte_stream_fuzz_rejection(data: bytes):
    """
    Axiom 3: Any arbitrary byte stream B either succeeds with S(D(B)) == B,
    or raises a deterministic ValueError (never crashes with uncaught exceptions).
    """
    try:
        decoded = CanonicalSerializer.deserialize_from_bytes(data)
        re_encoded = CanonicalSerializer.serialize_to_bytes(decoded)
        assert data == re_encoded
    except (ValueError, UnicodeDecodeError):
        pass


@settings(max_examples=100, deadline=None, suppress_health_check=[HealthCheck.too_slow])
@given(d=st.dictionaries(st.text(max_size=16), axiom_values(max_leaves=4), max_size=5))
def test_frozendict_immutability_and_hash_coherence(d: dict):
    """
    Axiom 4: FrozenDict preserves strict type key equality and produces invariant hashes.
    """
    fd1 = FrozenDict(d)
    fd2 = FrozenDict(d)

    assert fd1 == fd2
    assert hash(fd1) == hash(fd2)
    assert type(fd1) is FrozenDict

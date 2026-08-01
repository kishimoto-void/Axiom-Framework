# AXIOM COMMON PROTOCOL (ACP)

«RFC-Grade Deterministic State Coordinate & Causal Proof Protocol»

A language-neutral protocol for deterministic state identification, causal verification, and cryptographic proof across AI systems.

---

## Overview

**AXIOM Common Protocol (ACP)** is a language-independent protocol specification for representing immutable state, causal transitions, and cryptographic verification.

The protocol is designed for distributed AI systems, multi-agent reasoning, workflow engines, and any environment that requires deterministic state exchange across different implementations.

Unlike application-specific frameworks, ACP intentionally contains **no reasoning logic** and **no AI model assumptions**.

Its responsibility is limited to:

- Deterministic State Coordinates
- Causal DAG representation
- Replay protection
- Canonical serialization
- Cryptographic proof attachment
- Cross-language interoperability

---

## Design Philosophy

ACP follows several core principles.

### Language Neutral

The protocol is independent of Python and can be implemented in Rust, Go, C++, Java, TypeScript, or any language capable of producing canonical JSON.

### Intelligence Neutral

ACP does not describe reasoning.

It describes **state integrity**.

Reasoning engines remain free to evolve independently.

### Deterministic

Given identical input data, every implementation must produce:

- identical canonical JSON
- identical hashes
- identical Coordinate IDs

### Immutable Core

The protocol represents **immutable state coordinates**.

Runtime information belongs outside the protocol.

### Representation Independent

ACP does **not** depend on any particular state representation.

It can wrap:

- PLP particle states
- Robot poses
- Financial transactions
- Sensor readings
- LLM internal states
- Database snapshots
- Any other structured state

PLP is the first native state representation profile of ACP, not its foundation.

---

## Protocol Layers

```
Applications
         │
  LRP / PSS / DCK
         │
      Capsule
         │
  ┌──────┴──────┐
  ▼             ▼
 ACP           PLP
State Integrity   State Representation
  │             │
  └──────┬──────┘
         │
    Runtime / Reality
```

| Layer | Role | Question |
|-------|------|----------|
| **ACP** | State Integrity Layer | When, from where, how did it change, and who proved it? |
| **PLP** | State Representation Layer | What *is* the state? (particles, geometry, dynamics) |

---

## Core Components

### Header

Protocol metadata.

- `protocol`
- `version`
- `encoding`
- `hash algorithm`

### Genesis

Represents the unique origin of a state universe.

Provides:

- `genesis_id`
- `creator`
- `timestamp`
- `genesis_hash`
- `initial_state_hash`

### Core

Represents immutable state.

Contains:

- Identity
- State
- Invariant
- Constraint

The **Core Hash** uniquely identifies the state.

### Transition

Represents a causal state transition.

Supports:

- branching
- merging
- Lamport logical clocks
- replay protection

Transitions form a Directed Acyclic Graph (DAG).

### Proof

Cryptographic proof envelope.

Supports external signature verification via a pluggable verifier interface.

The protocol itself does not mandate a signature algorithm.

### Extension

Vendor-defined extension namespace.

Reserved protocol fields remain protected while allowing custom metadata.

---

## Security Features

ACP includes:

- RFC 8785 JSON Canonicalization (JCS)
- SHA-256 deterministic hashing
- Cryptographic domain separation
- Genesis anchoring
- Kahn's Algorithm DAG verification
- Lamport Logical Clock validation
- Transition ID uniqueness
- Replay protection
- Proof size limits
- Vendor namespace validation

---

## Relationship to Capsule

ACP intentionally stores only **immutable protocol information**.

Mutable execution data belongs in **Capsule**.

| ACP | Capsule |
|-----|---------|
| State Coordinate | Runtime State |
| Causal DAG | Observer Data |
| Identity | Reasoning Trace |
| Proof | Execution Result |
| Hashes | Model Output |

This separation keeps the protocol deterministic while allowing runtime flexibility.

---

## Relationship to PLP

PLP and ACP are **siblings**, not a hierarchy.

- **PLP** answers “what is the state?”
- **ACP** answers “how is the state identified, ordered, and proven?”

ACP can carry PLP states, but is deliberately independent of any single representation model.  
This keeps ACP adoptable by systems that do not use PLP.

---

## Reference Implementation

This repository contains the **Python reference implementation**.

Path: `src/axiom/common_protocol.py` (v1.0.3)

Other implementations are expected to produce identical canonical output.

Future reference implementations include:

- Rust
- Go
- Java
- C#
- TypeScript

---

## Conformance

An implementation is considered conformant if it produces identical:

- Canonical JSON
- Core Hash
- Genesis Hash
- Coordinate ID

for the official Golden Test Vectors.

---

## Versioning

ACP follows Semantic Versioning.

- **Patch** versions fix implementation issues without changing the protocol.
- **Minor** versions extend the protocol while maintaining compatibility.
- **Major** versions may introduce breaking protocol changes.

---

## License

**Non-Commercial / Non-Military Use Only.**

Commercial use is currently prohibited.

See the repository [LICENSE](../LICENSE) file for full terms.

---

## Status

| Item | Status |
|------|--------|
| Current Specification | AXIOM Common Protocol **v1.0.3** |
| Reference Implementation | Python |
| Specification Status | Stable Reference |

---

## Related Documents

- [AXIOM COMMON PROTOCOL Roadmap](AXIOM_COMMON_PROTOCOL_ROADMAP.md)
- [Source](../src/axiom/common_protocol.py)

AXIOM Common Protocol (ACP) v2.0

Protocol Constitution (Draft RFC)

Status

Draft

Scope

This document defines the constitutional protocol contract for all AXIOM Framework implementations.

ACP does not define application behavior.
ACP defines the invariant rules that every compliant implementation MUST satisfy.

---

Article 1 — Purpose

The purpose of ACP is to establish a deterministic, language-independent protocol for representing, sealing, verifying, and transporting AXIOM state.

ACP guarantees that equivalent input produces equivalent protocol output regardless of implementation language or execution environment.

---

Article 2 — Core Principles

Every ACP implementation SHALL satisfy the following principles.

2.1 Determinism

Identical canonical input MUST produce identical protocol output.

Execution order, memory layout, operating system, runtime, or programming language MUST NOT affect the result.

---

2.2 Observer Isolation

Observation MUST NOT modify protocol state.

Inspection, debugging, logging, monitoring, or visualization SHALL have zero semantic influence.

---

2.3 Canonical Representation

Every serializable object SHALL have exactly one canonical representation.

No implementation-specific formatting may affect hashes or proofs.

---

2.4 Language Independence

ACP SHALL define protocol contracts rather than implementation details.

Rust, Python, Go, C++, Java, JavaScript, or future implementations SHALL produce identical protocol results.

---

2.5 Projection Safety

Projection represents candidate interpretation only.

Projection MUST NEVER overwrite immutable protocol truth.

Semantic interpretation SHALL remain outside ACP constitutional authority.

---

Article 3 — Constitutional Capsule Model

Every Capsule SHALL contain the following constitutional sections.

Capsule
├── Header
├── Immutable Layer (A)
├── Projection Layer (B)
├── ACP Seal
└── Metadata

---

Immutable Layer (A)

The A layer represents protocol truth.

Properties:

- Immutable
- Deterministic
- Hash protected
- Canonical
- Protocol authoritative

No implementation may mutate A after sealing.

---

Projection Layer (B)

The B layer represents projected or candidate state.

Properties:

- Extendable
- Replaceable
- Versioned
- Non-authoritative

Projection SHALL NEVER invalidate A.

---

Article 4 — Hash Constitution

ACP defines constitutional hashes.

HashA

Protects Immutable Layer.

HashA = Hash(Canonical(A))

---

HashB

Protects Projection Layer.

HashB = Hash(Canonical(B))

---

Composite Hash

CompositeHash =
Hash(HashA || HashB || Header)

CompositeHash uniquely identifies the constitutional state.

---

Article 5 — Domain Separation

Every hash operation SHALL include explicit domain separation.

Example:

ACP::A
ACP::B
ACP::SEAL
ACP::HEADER
ACP::PROOF

No domain collision is permitted.

---

Article 6 — ACP Seal

Every finalized Capsule SHALL contain an ACP Seal.

The seal certifies:

- protocol version
- hash algorithm
- canonical serialization version
- HashA
- HashB
- CompositeHash

Without a valid ACP Seal, a Capsule SHALL NOT be considered constitutionally valid.

---

Article 7 — Proof Chain

Every protocol transition SHALL be verifiable.

Input
 ↓
Canonical State
 ↓
HashA
 ↓
HashB
 ↓
CompositeHash
 ↓
ACP Seal
 ↓
Verification

Each step MUST be reproducible.

---

Article 8 — Version Constitution

ACP versions SHALL follow explicit compatibility rules.

Major version:

- constitutional change

Minor version:

- backward-compatible extension

Patch version:

- clarification or correction

A major version MUST NOT silently reinterpret previous constitutional meaning.

---

Article 9 — Constitutional Compliance

A compliant implementation MUST:

- preserve determinism
- preserve canonical serialization
- preserve hash equality
- preserve proof reproducibility
- preserve observer isolation

Failure of any item results in constitutional non-compliance.

---

Article 10 — Golden Conformance

Every ACP implementation SHALL pass the official Golden Vector suite.

Cross-language equality is mandatory.

Rust = Python = Go = C++ = Java = JavaScript

Protocol equality takes precedence over implementation convenience.

---

Article 11 — Future Extensions

Extensions MAY introduce new Projection capabilities.

Extensions MUST NOT alter:

- Immutable Layer
- Hash Constitution
- Canonical Serialization
- Proof Chain
- Observer Isolation

Future evolution SHALL extend the Constitution rather than replace it.

---

Constitutional Statement

ACP is not an application framework.

ACP is the constitutional contract that guarantees deterministic protocol identity across every AXIOM implementation.

All higher-level systems—including Capsule, PLP, DCK, PSS, and future UPR stages—derive their interoperability and trust from this constitutional foundation.

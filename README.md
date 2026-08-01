# Axiom Framework

«A Universal Protocol Framework for Intelligence-Neutral AI Systems»

Axiom Framework is a protocol-based architecture for constructing AI systems through deterministic state transitions rather than model-specific implementations.

Instead of defining how an AI should think, Axiom Framework defines how **reasoning, state, and execution** should be represented.

The framework is designed to be:

- Intelligence Neutral
- Language Neutral
- Runtime Neutral
- Vendor Independent
- Deterministic by Design

The objective is to establish an open protocol layer that enables different AI models, runtimes, and programming languages to interoperate through common specifications.

---

## Core Philosophy

Traditional AI frameworks tightly couple:

- reasoning
- execution
- memory
- prompts
- model implementation

Axiom Framework separates these concerns into independent protocol layers.

```
Application
      │
Reasoning Engine (LRP / PSS / DCK)
      │
Capsule (Mutable Runtime Payload)
      │
──────────────────────────────────
AXIOM Common Protocol (ACP)
(State Coordinate / Causal DAG / Proof / Hash)
──────────────────────────────────
      │
Physical / Logical State (PLP)
```

Each protocol owns a single responsibility.  
This enables replacement of individual components without redesigning the overall architecture.

---

## AXIOM Common Protocol (ACP)

**ACP** is the immutable core of the framework.

It is an RFC-grade, language-neutral protocol for:

- Deterministic State Coordinates
- Causal DAG representation
- Replay protection
- Canonical serialization (RFC 8785 JCS)
- Cryptographic proof attachment
- Cross-language interoperability

ACP intentionally contains **no reasoning logic** and **no AI model assumptions**.

| ACP (Immutable) | Capsule (Mutable) |
|-----------------|-------------------|
| State Coordinate | Runtime State |
| Causal DAG | Observer Data |
| Identity | Reasoning Trace |
| Proof | Execution Result |
| Hashes | Model Output |

**Current Version**: v1.0.3  
**Reference Implementation**: Python  
**Status**: Stable Reference + Golden Test Vectors (10/10 PASS)

### Key Documents

- [ACP Overview](docs/AXIOM_COMMON_PROTOCOL.md)
- [ACP Roadmap](docs/AXIOM_COMMON_PROTOCOL_ROADMAP.md)
- [Golden Test Vectors](tests/vectors/README.md)
- [Conformance Report](tests/vectors/CONFORMANCE_REPORT.md)

---

## Architecture

```
Applications
                     │
     ┌───────────────┼───────────────┐
     │               │               │
    LRP            PSS            DCK
     │               │               │
     └───────────────┼───────────────┘
                     │
                Capsule Standard
           （可変・実行時ペイロード）
                     │
────────────────────────────────────────
                     │
        AXIOM COMMON PROTOCOL
（不変の状態座標・因果DAG・Proof・Hash）
                     │
────────────────────────────────────────
                     │
                    PLP
      （言語・モデル非依存の状態表現）
```

---

## Components

### Universal Protocol Runtime (UPR)

The execution foundation of the framework.

Responsibilities:

- protocol orchestration
- state transition
- event routing
- runtime lifecycle
- module interoperability

UPR intentionally contains no intelligence.  
It only manages protocol execution.

### PLP (Particle Language Protocol)

PLP represents information as semantic-independent particles.

Instead of storing language, PLP stores structured state.

Features:

- language independent
- deterministic
- model neutral
- observer independent

### Capsule

Capsules package **mutable** runtime information.

A Capsule may include:

- reasoning context
- observer data
- execution results
- embeddings
- memory traces

Capsules are portable between runtimes and sit **above** ACP.

### PSS (Problem Specification System)

Defines problems before reasoning begins.

PSS separates:

- goals
- constraints
- assumptions
- evaluation criteria

### LRP (LLM Reasoning Protocol)

Defines reasoning as observable state transitions.

Rather than storing conversations, LRP stores transition history.

This enables replay, auditing, and deterministic inspection.

---

## Design Principles

- Intelligence Neutral
- Protocol First
- State Transition First
- Deterministic Execution
- Replaceable Modules
- Language Independence
- Runtime Independence
- Observable Reasoning
- Minimal Core
- Extensible Architecture
- **Immutable Core / Mutable Extension** (ACP ↔ Capsule)

---

## Repository Structure

```
Axiom-Framework/
├── README.md
├── ROADMAP.md
├── LICENSE                          # Non-Commercial / Non-Military
│
├── docs/
│   ├── AXIOM_COMMON_PROTOCOL.md
│   ├── AXIOM_COMMON_PROTOCOL_ROADMAP.md
│   └── UPR_v1.2_Specification.md
│
├── src/
│   ├── axiom/
│   │   ├── common_protocol.py       # ACP v1.0.3 Reference Implementation
│   │   └── upr.py
│   └── modules/
│       ├── plp_kernel.py
│       └── plp_capsule.py
│
└── tests/
    └── vectors/                     # Official Golden Test Vectors
        ├── README.md
        ├── CONFORMANCE_REPORT.md
        ├── *.json
        └── expected/
```

---

## Current Modules

| Module | Status |
|--------|--------|
| **AXIOM Common Protocol (ACP) v1.0.3** | **Stable Reference** |
| Golden Test Vectors | **10/10 PASS** |
| UPR v1.2 | Stable |
| PLP Kernel | Stable |
| PLP Capsule | Stable |
| PSS | In Progress |
| LRP | In Progress |
| DCK | Planned Integration |

---

## Goals

Axiom Framework aims to become an **open protocol specification** rather than a traditional software framework.

The long-term objective is interoperability between:

- local LLMs
- cloud AI
- robotics
- distributed systems
- multi-agent systems
- future reasoning engines

without changing the protocol layer.

ACP is designed so that independent implementations in Rust, Go, Java, TypeScript, etc. can achieve bit-identical results against the Golden Test Vectors.

---

## License

**Non-Commercial / Non-Military Use Only.**

Commercial use is currently prohibited.  
See the [LICENSE](LICENSE) file for full terms.

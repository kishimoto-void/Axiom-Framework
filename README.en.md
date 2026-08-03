# Axiom Framework

**A Universal Protocol Framework for Intelligence-Neutral AI Systems**

*“Same input — yet the state drifts just because the language or implementation differs.”*  
That drift becomes a breeding ground for inconsistency and hallucination across LLMs and multi-agent systems.

Axiom Framework is a **language-neutral, intelligence-neutral, deterministic protocol layer** designed to eliminate that problem at the root.

Instead of defining *how* an AI should think, Axiom defines *how state is represented, verified, and constrained* — so different languages, runtimes, and models can share the same reality.

**Latest**: [v1.1.0](https://github.com/kishimoto-void/Axiom-Framework/releases/tag/v1.1.0) · Rust companion: [`axiomFrameworkRUSTv1.5/`](axiomFrameworkRUSTv1.5/)  
Japanese README: [README.md](README.md)

---

## Why This Exists

In conventional AI stacks, the same input often yields subtle differences:

- Serialization diverges between Python, Rust, and C  
- Timestamps and floating-point handling become implementation-dependent  
- Hashes and state IDs fail to match  
- You end up with states that *look* the same but *are not* the same  

When those differences accumulate, multi-agent systems lose the ability for one LLM to correctly **verify and constrain** another LLM’s output.

Axiom Framework was built to erase that invisible drift.

---

## Design Philosophy (Core Uniqueness)

### 1. Clear separation of immutable and mutable

| Layer | Role | Nature |
|-------|------|--------|
| **ACP** (AXIOM Common Protocol) | Integrity, causality, proof | **Immutable** |
| **PLP Capsule** | Runtime state, observation, growth | **Mutable** |

- The **immutable core (ACP)** guarantees bit-identical results across languages and implementations  
- All **growth and change** is confined to Capsules  
- This cleanly separates “constraints that must never break” from “parts that are allowed to evolve”  

### 2. Intelligence Neutral

Axiom does not define *how to think*.  
It only defines *how to express state and how to prove it*.

- No LLM inference logic  
- No model-specific assumptions  
- A shared foundation usable by any LLM or agent  

### 3. Mutual constraint monitoring in multi-agent systems

The goal is a structure where immutable constraints (e.g. “do not attack other programs”) can be **verified and enforced by LLMs against other LLMs**.

- Round-robin rotation of monitoring LLMs  
- Time-offset resets to limit corruption  
- Flexible enough for enterprise black-box deployments  

### 4. Making drift reduction *visible*

Technical elimination of drift is not enough.  
Anyone must be able to *see* that there is no drift.

- **Golden Vectors** proving cross-language hash identity  
- Hand-written deterministic serializers (no dependence on language-default JSON)  
- Details such as string-fixed `timestamp_ns` and float canonicalization, end-to-end determinism  

> “If it cannot be seen, no one will trust it.”  
> That is why drift reduction is published in a **provable** form.

---

## Architecture

```
Applications
     │
  LRP / PSS / DCK
     │
  Capsule (mutable / growth)
     │
┌────┴────┐
▼         ▼
ACP       PLP
State     State
Integrity Representation
(immutable) (representation)
     │
Runtime / Reality
```

| Layer | Role |
|-------|------|
| **ACP** | When, from where, how it changed, and who proved it |
| **PLP** | What the state *is* (particles, geometry, dynamics) |
| **Capsule** | Mutable runtime state, observations, growth |
| **PSS** | Problem specification |
| **DCK** | Shared substrate / difference convergence |
| **LRP** | Reasoning as observable state transitions |
| **UPR** | Protocol execution runtime (no intelligence) |

ACP can wrap any state (robot pose, transaction, sensor, LLM internal state, …).  
PLP is the first native state-representation profile of ACP.

---

## Current Status

| Module | Status |
|--------|--------|
| **ACP v1.1.0** | Stable Reference (Golden Vectors 10/10 PASS) |
| **PLP Capsule v1.1.3** | Stable (Rust / Python cross-language identity verified) |
| **UPR v1.2** | Stable |
| PSS / LRP / DCK | In progress |

Notable result:

- **Byte-identical Canonical Hash** for the same input in Rust and Python  
- Language-induced state drift eliminated at the protocol level  

### Rust companion

[`axiomFrameworkRUSTv1.5/`](axiomFrameworkRUSTv1.5/) contains:

- `plp_capsule_v1_1_3.rs` — hand-written deterministic serializer + fixed Golden Hash  
- `acp_v1_1_0_reference.rs` — ACP normative reference  
- `PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md` — 10 cases (empty, multi-observer, Added/Modified/Removed, Japanese, control characters)  

---

## Key Documents

- [ACP SPECIFICATION (RFC-AXIOM-0001) v1.1.0](docs/AXIOM_COMMON_PROTOCOL_SPECIFICATION.md)
- [ACP Overview](docs/AXIOM_COMMON_PROTOCOL.md)
- [ACP Roadmap](docs/AXIOM_COMMON_PROTOCOL_ROADMAP.md)
- [Golden Test Vectors (ACP)](tests/vectors/README.md)
- [PLP Capsule Golden Vectors v1.1.3](axiomFrameworkRUSTv1.5/PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md)
- [UPR v1.2](docs/UPR_v1.2_Specification.md)

---

## Repository Structure

```
Axiom-Framework/
├── README.md                      # Japanese (primary)
├── README.en.md                   # English
├── ROADMAP.md
├── LICENSE
├── docs/                          # ACP / UPR specs
├── src/
│   ├── axiom/                     # ACP + UPR (Python)
│   └── modules/                   # PLP kernel / capsule, DCK, …
├── tests/vectors/                 # ACP Golden Vectors
└── axiomFrameworkRUSTv1.5/        # Rust reference + Capsule Golden Vectors
```

---

## Goals

Not merely a “convenient library,” but a **shared substrate** where different languages, different LLMs, and different agents can hold the same state and obey the same constraints.

1. Eliminate physical drift (hashes, serialization)  
2. Make semantic drift visible and gradually shrink it  
3. Provide a structure in which immutable constraints can be mutually monitored by LLMs  

---

## License

**[AXIOM Framework Research License v1.0](LICENSE)**

- Personal, academic, educational, and non-profit use permitted  
- Attribution required  
- Military and harm-oriented use prohibited  
- Commercial use requires a separate license  
- DCK is under MIT (separate component)  

---

**Axiom Framework**  
Fix the immutable in the protocol; allow growth in the Capsule.  
Toward a common state foundation beyond language and intelligence.

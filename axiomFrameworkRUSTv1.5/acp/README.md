# axiom-acp — ACP v1.1.0 Normative Reference (Rust)

AXIOM Common Protocol (ACP) v1.1.0 reference implementation.

- Pure **RFC 8785 JCS** canonicalization
- Multi-hash engine: SHA-256 / SHA3-256 / BLAKE3
- Deterministic State Coordinate / Causal DAG / Proof protocol
- ISO-8601 UTC canonicalization

Specification: RFC-AXIOM-0001 (Candidate)

## Build & Test

```bash
cd axiomFrameworkRUSTv1.5/acp
cargo test
```

## Usage

```rust
use axiom_acp::*;
// See lib.rs module docs and tests for full API surface
```

Part of **Axiom-Framework** Rust v1.5 reference suite.

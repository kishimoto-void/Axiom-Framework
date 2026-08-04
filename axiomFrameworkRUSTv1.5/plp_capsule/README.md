# plp-capsule — PLP Capsule v1.1.3

Production-ready reference implementation of the PLP Capsule.

- Hand-written deterministic serialization
- `timestamp_ns` always serialized as decimal string (JS 53-bit safety)
- SHA-256 (default) / optional BLAKE3
- Golden hash vectors aligned with Python

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `sha2-hash` | yes | SHA-256 hashing |
| `blake3-hash` | no | BLAKE3 hashing |

## Setup (one-time)

Full source currently lives at the sibling file. Copy it into the crate:

```bash
cp axiomFrameworkRUSTv1.5/plp_capsule_v1_1_3.rs axiomFrameworkRUSTv1.5/plp_capsule/src/lib.rs
cargo test --manifest-path axiomFrameworkRUSTv1.5/plp_capsule/Cargo.toml
```

## Build & Test

```bash
cd axiomFrameworkRUSTv1.5/plp_capsule
cargo test
cargo test --features blake3-hash
```

Part of **Axiom-Framework** Rust v1.5 reference suite.

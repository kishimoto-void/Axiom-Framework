# Axiom Framework — Rust v1.5

Rust reference implementations for the Axiom / PLP / LRP / PSS / DCK / ACP ecosystem.

**Status:** PLP Capsule + ACP complete (now as crates). LRP v1.5.0 research snapshot. PSS scaffolding. DCK modular + nalgebra v2.0.

## Contents

| Path | Description |
|------|-------------|
| `acp/` | **ACP v1.1.0** crate (`axiom-acp`) — JCS / multi-hash / causal DAG |
| `acp_v1_1_0_reference.rs` | Full ACP source (copy into `acp/src/lib.rs` if not yet migrated) |
| `plp_capsule/` | **PLP Capsule v1.1.3** crate (`plp-capsule`) |
| `plp_capsule_v1_1_3.rs` | Full PLP source (copy into `plp_capsule/src/lib.rs` if not yet migrated) |
| `dck/` | **DCK v2.0** modular + nalgebra multi-D crate |
| `lrp/` | **LRP v1.5.0-research-rust-strict-final** (provisional) |
| `pss/` | **PSS v1.0.0-rc1** scaffolding |
| `PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md` | Official Golden Vectors |
| `PLP_CAPSULE_GOLDEN_TEST_REPORT.md` | Cross-language test report |
| `LRP_GOLDEN_VECTORS_v1_5_provisional.md` | LRP quantitative notes |
| `dck/DCK_PYTHON_RUST_GOLDEN_VECTOR_COMPARISON.md` | Python ↔ Rust DCK comparison |

## Crates (Cargo)

```bash
# ACP
cargo test --manifest-path axiomFrameworkRUSTv1.5/acp/Cargo.toml

# PLP Capsule
cargo test --manifest-path axiomFrameworkRUSTv1.5/plp_capsule/Cargo.toml

# DCK
cargo run --manifest-path axiomFrameworkRUSTv1.5/dck/Cargo.toml

# LRP
cargo test --manifest-path axiomFrameworkRUSTv1.5/lrp/Cargo.toml

# PSS
cargo test --manifest-path axiomFrameworkRUSTv1.5/pss/Cargo.toml
```

### One-time source copy (ACP / PLP)

Large reference sources are still kept as top-level `.rs` for readability.  
To make the crates fully buildable:

```bash
# ACP — strip leading markdown if present, keep from first `//!` / `use`
cp axiomFrameworkRUSTv1.5/acp_v1_1_0_reference.rs axiomFrameworkRUSTv1.5/acp/src/lib.rs
# edit if needed to remove markdown wrapper

# PLP Capsule (already pure Rust)
cp axiomFrameworkRUSTv1.5/plp_capsule_v1_1_3.rs axiomFrameworkRUSTv1.5/plp_capsule/src/lib.rs
```

## LRP v1.5.0 (Provisional)

- 100% Deterministic Reasoning Session Engine
- Observer Isolation via `std::panic::catch_unwind`
- Strict DeltaAction × DeltaKind matching
- Full PLP-compatible delta application + Replay / Fork
- No wall-clock, no Uuid::v4, BTreeMap only
- Quantitative Validation First

> **Note:** 1.5 完成時に調整します。現時点は research snapshot として配置。

### Layout

```
axiomFrameworkRUSTv1.5/
├── README.md
├── acp_v1_1_0_reference.rs      # full ACP source
├── plp_capsule_v1_1_3.rs        # full PLP source
├── acp/                         # crate: axiom-acp
├── plp_capsule/                 # crate: plp-capsule
├── dck/                         # crate: dck_modular
├── lrp/
├── pss/
└── *.md                         # golden reports
```

Ready as the cross-language reference companion to the Python Axiom-Framework modules.

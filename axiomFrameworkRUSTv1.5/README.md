# Axiom Framework — Rust v1.5

Rust reference implementations for the Axiom / PLP / LRP / PSS ecosystem.

**Status:** PLP Capsule + ACP complete. LRP v1.5.0 research snapshot added (provisional). PSS scaffolding added.

## Contents

| File / Dir | Description |
|------------|-------------|
| `plp_capsule_v1_1_3.rs` | **PLP Capsule v1.1.3** production reference |
| `acp_v1_1_0_reference.rs` | **ACP v1.1.0** normative reference (JCS / multi-hash / causal DAG) |
| `PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md` | Official Golden Vectors (10 cases, Rust ≡ Python) |
| `PLP_CAPSULE_GOLDEN_TEST_REPORT.md` | Cross-language test report |
| `lrp/` | **LRP v1.5.0-research-rust-strict-final** (provisional snapshot) |
| `LRP_GOLDEN_VECTORS_v1_5_provisional.md` | LRP quantitative Golden notes (to be finalized at 1.5 completion) |
| `pss/` | **PSS v1.0.0-rc1** Problem Specification Standard (scaffolding / 本体手動追加) |

## LRP v1.5.0 (Provisional)

- 100% Deterministic Reasoning Session Engine
- Observer Isolation via `std::panic::catch_unwind`
- Strict DeltaAction × DeltaKind matching
- Full PLP-compatible delta application + Replay / Fork
- No wall-clock, no Uuid::v4, BTreeMap only
- Quantitative Validation First

> **Note:** 1.5 完成時に調整します。現時点は research snapshot として配置。

### Layout (updated)

```
axiomFrameworkRUSTv1.5/
├── README.md
├── plp_capsule_v1_1_3.rs
├── acp_v1_1_0_reference.rs
├── PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md
├── PLP_CAPSULE_GOLDEN_TEST_REPORT.md
├── LRP_GOLDEN_VECTORS_v1_5_provisional.md
├── lrp/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── main.rs
└── pss/
    ├── Cargo.toml
    ├── README.md
    └── src/
        ├── lib.rs
        └── main.rs
```

Ready as the cross-language reference companion to the Python Axiom-Framework modules.

# Axiom Framework — Rust v1.5

Rust reference implementations for the Axiom / PLP ecosystem.

**Status: complete** (sources + golden vectors pushed)

## Contents

| File | Size | Description |
|------|------|-------------|
| `plp_capsule_v1_1_3.rs` | ~38 KB | **PLP Capsule v1.1.3** production reference |
| `acp_v1_1_0_reference.rs` | ~30 KB | **ACP v1.1.0** normative reference (JCS / multi-hash / causal DAG) |
| `PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md` | — | Official Golden Vectors (10 cases, Rust ≡ Python) |
| `PLP_CAPSULE_GOLDEN_TEST_REPORT.md` | — | Cross-language test report |

## PLP Capsule v1.1.3

- Hand-written deterministic serializer (no `serde_json::to_vec` lock-in)
- `timestamp_ns` always a decimal **string** in Canonical Hash (JS 53-bit safe)
- `write_raw_str` iterates Unicode scalar values (`chars()`)
- `DeltaKind` independent of Debug (`added` / `modified` / `removed`)
- Clear integrity API: `verify` · `recompute_hash` · `seal`
- `BuildParams` + `build_with_meta` for deterministic tests / Golden Vectors
- Empty `CapabilityRegistry` by default (register only what you need)

### Protocol / version

- Protocol: `PLP/1.1`
- Capsule version: `1.1.3`
- Bundle label: **axiomFrameworkRUSTv1.5** (2026-08)

### Fixed Golden Hash (case 02_single_obs)

```
a54b533ae4223cbef6d6227c957ac22d11efbcf61deead82bbc1e17134c3941e
```

Full suite: **10 / 10 PASS** (see `PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md`).

### Recommended Cargo.toml

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
thiserror = "1"
ryu = "1"
hex = "0.4"
sha2 = "0.10"
uuid = { version = "1", features = ["v4"] }
# optional
blake3 = { version = "1", optional = true }

[features]
default = ["sha2-hash"]
sha2-hash = []
blake3-hash = ["blake3"]
```

## ACP v1.1.0

Normative reference for AXIOM Common Protocol:

- RFC 8785 JCS (UTF-16 code unit key ordering)
- Multi-hash: SHA-256 / SHA3-256 / BLAKE3
- Causal DAG verification (topo sort, Lamport, single-root)
- Domain separation tags

> Note: the uploaded `acp_v1_1_0_reference.rs` includes a short Japanese change-log preface before the `//! AXIOM Common Protocol` module header. The executable Rust body starts at the `use chrono::...` / `//! AXIOM` block.

## Layout

```
axiomFrameworkRUSTv1.5/
├── README.md
├── plp_capsule_v1_1_3.rs
├── acp_v1_1_0_reference.rs
├── PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md
└── PLP_CAPSULE_GOLDEN_TEST_REPORT.md
```

Ready as the cross-language reference companion to the Python Axiom-Framework / PLP modules.

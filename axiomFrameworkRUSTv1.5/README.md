# Axiom Framework — Rust v1.5

Rust reference implementations for the Axiom / PLP ecosystem.

## Contents

| File | Description |
|------|-------------|
| `plp_capsule_v1_1_3.rs` | **PLP Capsule v1.1.3** — production-ready reference with fully controlled deterministic serialization, ryu float canonicalization, clear integrity API (`verify` / `recompute_hash` / `seal`), Golden Vector suite (10/10 cross-language match with Python) |
| `PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md` | Official Golden Vectors (10 cases: empty, multi-observer, Added/Modified/Removed, Japanese, control characters) |
| `acp_v1_1_0_reference.rs` | AXIOM Common Protocol (ACP) v1.1.0 normative reference (JCS, multi-hash, causal DAG) |

## PLP Capsule v1.1.3 highlights

- Hand-written deterministic serializer (no `serde_json::to_vec` lock-in)
- `timestamp_ns` always string in Canonical Hash (JS 53-bit safe)
- `write_raw_str` uses Unicode scalar values (`chars()`)
- `DeltaKind` independent of Debug
- `BuildParams` + `build_with_meta` for deterministic tests
- Cross-language Golden Hash verified (Rust ≡ Python)

### Recommended Cargo features

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
thiserror = "1"
ryu = "1"
hex = "0.4"
sha2 = "0.10"
uuid = { version = "1", features = ["v4"] }

[features]
default = ["sha2-hash"]
sha2-hash = []
blake3-hash = ["blake3"]
```

## Version alignment

- Protocol: `PLP/1.1`
- Capsule version: `1.1.3`
- Bundle label: **axiomFrameworkRUSTv1.5** (2026-08)

## Status

Ready as cross-language reference for PLP Capsule hashing and as a companion to the Python Axiom-Framework / PLP modules.

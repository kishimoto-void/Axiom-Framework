# Axiom Framework — Rust v1.5

Rust reference bundle for the Axiom / PLP ecosystem (2026-08).

## Status

| Asset | Status |
|-------|--------|
| **Golden Vectors v1.1.3** | ✅ Pushed — 10/10 cross-language PASS (Rust ≡ Python) |
| **PLP Capsule v1.1.3 source** | ⏳ Full `.rs` (~38KB) prepared locally; upload in progress / manual |
| **ACP v1.1.0 reference** | ⏳ Full `.rs` prepared locally |

## Files in this directory

- `PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md` — Official golden suite (empty, multi-observer, Added/Modified/Removed, Japanese, control chars)
- `README.md` — this file

## PLP Capsule v1.1.3 design summary

- Hand-written deterministic serializer (no `serde_json::to_vec` lock-in)
- `timestamp_ns` always string in Canonical Hash (JS 53-bit safe)
- `write_raw_str` via Unicode scalar values (`chars()`)
- `DeltaKind` independent of Debug
- `verify` / `recompute_hash` / `seal` clear contracts
- `BuildParams` + `build_with_meta` for deterministic tests

### Protocol / version

- Protocol: `PLP/1.1`
- Capsule version: `1.1.3`
- Bundle: **axiomFrameworkRUSTv1.5**

### Recommended Cargo.toml

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

## Local artifacts (Grok sandbox)

```
/home/workdir/artifacts/plp_capsule_v1_1_3.rs
/home/workdir/artifacts/acp_v1_1_0_reference.rs
/home/workdir/artifacts/PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md
```

Full source push of the large `.rs` files may require a follow-up commit (tool payload limits). Golden Vectors are authoritative and already live.

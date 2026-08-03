# Extract Rust sources

```bash
base64 -d rust_sources.zip.b64 > rust_sources.zip
unzip rust_sources.zip
```

Produces:
- `plp_capsule_v1_1_3.rs` (PLP Capsule v1.1.3 production reference)
- `acp_v1_1_0_reference.rs` (ACP v1.1.0 normative reference)

Golden Vectors: see `PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md` (10/10 PASS).

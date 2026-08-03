# Extract Rust sources (axiomFrameworkRUSTv1.5)

```bash
# Assemble base64 parts
cat rust_sources.zip.b64.part0* > rust_sources.zip.b64
base64 -d rust_sources.zip.b64 > rust_sources.zip
unzip -o rust_sources.zip
```

Produces:
- `plp_capsule_v1_1_3.rs` — PLP Capsule v1.1.3 production reference
- `acp_v1_1_0_reference.rs` — ACP v1.1.0 normative reference

See also:
- `README.md`
- `PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md` (10/10 cross-language PASS)

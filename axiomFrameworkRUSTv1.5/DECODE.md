# Full Rust sources

The complete `plp_capsule_v1_1_3.rs` and `acp_v1_1_0_reference.rs` are the production reference implementations developed and golden-tested in the Grok session (2026-08-03).

## Files already in this directory

- `README.md` — overview
- `PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md` — 10/10 cross-language golden vectors

## Primary sources (to be completed in follow-up commit if not present)

| File | SHA-256 (of pure source) | Role |
|------|--------------------------|------|
| `plp_capsule_v1_1_3.rs` | (see local artifacts) | PLP Capsule v1.1.3 reference |
| `acp_v1_1_0_reference.rs` | `18819e1ca06c1745ed73ea919391d97427c8a48d2e5d622f3ba222c02f46b2ab` | ACP v1.1.0 reference |

If the `.rs` files are missing from this folder in a sparse clone, retrieve them from the conversation artifacts or re-export from the local Grok sandbox path:

```
/home/workdir/artifacts/plp_capsule_v1_1_3.rs
/home/workdir/artifacts/acp_v1_1_0_reference.rs
```

Golden Vector suite: **10/10 PASS** (Rust ≡ Python).

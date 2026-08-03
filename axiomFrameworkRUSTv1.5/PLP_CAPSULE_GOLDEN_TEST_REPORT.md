# PLP Capsule Golden Hash / Golden Vector Test Report

**Date:** 2026-08-03  
**Scope:** Cross-language verification of PLP Capsule Canonical Hash (v1.1.3 rules)  
**Rust reference:** `/home/workdir/artifacts/plp_capsule_v1_1_3.rs`  
**Python side:** Minimal re-implementation of the same deterministic serializer rules (Axiom-Framework / PLP repo existing capsule is a different generation)

---

## 1. Summary

| Item | Result |
|------|--------|
| Rust Golden Hash computation | **PASS** |
| Python Golden Hash (same rules) | **PASS** |
| Cross-language hash identity | **PASS** (`MATCH=True`) |
| Existing Axiom-Framework ACP vectors | Present (separate protocol) |
| Existing PLP Capsule v1.0 golden vectors (repo) | Present (older serde_json rules; not compatible with v1.1.3) |

**Conclusion:** Under the locked v1.1.3 deterministic serialization rules, Rust and Python produce **byte-identical** canonical payloads and therefore **identical SHA-256 hashes**.

---

## 2. Fixed Golden Input (v1.1.3)

```
protocol          = "PLP/1.1"
capsule_schema    = "v1/capsule"
version           = "1.1.3"
capsule_id        = "00000000-0000-4000-8000-000000000001"
parent_id         = null
clock             = 42
sequence          = 7
timestamp_ns      = "1700000000000000000"   # always string in Canonical Hash
source            = "golden"
is_keyframe       = true
hash_algorithm    = "sha256"

Observation (single):
  name         = "geom"
  schema       = "v1/geometry"
  capability   = "geometry"
  observer_id  = "cam0"
  values:
    x = 1.0   → canonical string "1.0"
    y = -2.5  → canonical string "-2.5"

delta = {}   (empty)
```

---

## 3. Canonical Bytes (identical in Rust & Python)

```
{"header":{"protocol":"PLP/1.1","capsule_schema":"v1/capsule","version":"1.1.3","capsule_id":"00000000-0000-4000-8000-000000000001","parent_id":null,"clock":42,"sequence":7,"timestamp_ns":"1700000000000000000","source":"golden","is_keyframe":true,"hash_algorithm":"sha256"},"observations":[{"name":"geom","schema":"v1/geometry","capability":"geometry","observer_id":"cam0","values":{"x":"1.0","y":"-2.5"}}],"delta":{}}
```

---

## 4. Golden Hash (SHA-256, lowercase hex)

```
a54b533ae4223cbef6d6227c957ac22d11efbcf61deead82bbc1e17134c3941e
```

| Implementation | Hash | Match |
|----------------|------|-------|
| Rust (v1.1.3 harness) | `a54b533ae4223cbef6d6227c957ac22d11efbcf61deead82bbc1e17134c3941e` | — |
| Python (same rules) | `a54b533ae4223cbef6d6227c957ac22d11efbcf61deead82bbc1e17134c3941e` | **True** |

---

## 5. Serialization Rules Locked for v1.1.3

1. **Hand-written deterministic serializer** (no `serde_json::to_vec` / no language-default JSON).
2. Field order is fixed in code (`header` → `observations` → `delta`).
3. All floating-point values are pre-canonicalized with ryu-style rules (`-0.0` → `"0"`, exponent normalized).
4. `timestamp_ns` is **always a decimal string** (never a JSON number) to avoid JS 53-bit issues.
5. `write_raw_str` iterates over **Unicode scalar values** (`chars()` / Python `for ch in s`).
6. Control characters (`< U+0020`) are escaped as `\u00XX`.
7. Object keys in `values` and `delta` follow sorted-map order (BTreeMap equivalent).
8. `DeltaKind` uses stable strings: `"added"` / `"modified"` / `"removed"` (never Debug).

---

## 6. Relationship to Existing Repo Artifacts

### Axiom-Framework (`kishimoto-void/Axiom-Framework`)

- Contains **ACP (AXIOM Common Protocol)** golden vectors under `tests/vectors/` (minimal, genesis, transition, merge, …).
- Contains an older **PLP Capsule Golden Vectors** document (`tests/PLP_CAPSULE_GOLDEN_VECTORS.md`) based on:
  - protocol `PLP/1.0`, version `1.0.0`
  - `serde_json::to_vec` style
  - different hash payload shape
- Those vectors remain valid for the **previous generation** and must **not** be mixed with v1.1.3 hashes.

### PLP repo (`kishimoto-void/PLP`)

- `plp_capsule.py` is **v1.3** of a different design (shorter hash, different fields, `json.dumps(sort_keys=True)`).
- It does **not** implement the v1.1.3 Canonical Hash rules.
- Cross-language identity for the new rules requires a dedicated Python port of the deterministic serializer (as demonstrated above).

---

## 7. Recommended Next Steps

1. Port the full v1.1.3 deterministic serializer into `Axiom-Framework` / `PLP` as the official Python reference.
2. Add the fixed golden vector above into `tests/PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md` (or equivalent).
3. Expand golden vectors:
   - empty observations
   - multi-observation sorted order
   - non-trivial delta (Added / Modified / Removed)
   - non-ASCII keys / values (to exercise `chars()` path)
4. Keep ACP vectors and PLP Capsule vectors in separate namespaces to avoid confusion.

---

## 8. Files Produced / Referenced

| Path | Role |
|------|------|
| `/home/workdir/artifacts/plp_capsule_v1_1_3.rs` | Rust production-ready reference |
| `/home/workdir/artifacts/PLP_CAPSULE_GOLDEN_TEST_REPORT.md` | This report |
| `/tmp/plp_golden` | Rust harness used to compute golden hash |
| `/tmp/plp_py_golden/golden_hash.py` | Minimal Python mirror of the same rules |

---

**Status:** Golden Hash cross-language verification for PLP Capsule v1.1.3 **succeeded**.

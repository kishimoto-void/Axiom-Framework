# PLP-R Golden Vectors v0.1.2

**Status**: **LOCKED** (2026-08-09)  
**Projector**: MinimalProjector (placeholder rules — Canonical Projection Candidates only)  
**Hash policy**: Dual Hash (`canonical_hash` + `raw_hash`)  
**Reference lock**: Python `golden_lock_ref.py` (serialization rules mirrored from Rust)

---

## Fixed Header Parameters

```
protocol       = "PLP-PROJ/0.1"
version        = "0.1.1"          # hash-relevant; do not change without re-lock
clock          = 42
sequence       = 0
capsule_id     = "golden-{id}"
timestamp_ns   = 1700000000000000000
source         = "golden"
hash_algorithm = "sha256"
parent_id      = null
```

---

## Locked Vectors

### 01_en_cat_sleep

| Field | Value |
|-------|-------|
| input | `cat sleeps on table` |
| language | `en` |
| tokens | `["cat", "sleeps", "on", "table"]` |
| annotations | ENTITY(cat/e1), ACTION(sleep/a1), LOCATION(table/l1) |
| **canonical_hash** | `b130e1ff7e86f406d8acdd5b720b5a350111a99c78ca3037bc8387dadabe629b` |
| **raw_hash** | `5dd6fc5d0435bf3e4911d369c9fdeede0463c0c4312ef663892692e74063c039` |

### 02_ja_cat_sleep

| Field | Value |
|-------|-------|
| input | `猫が机の上で寝ている。` |
| language | `ja` |
| tokens | `["猫が机の上で寝ている"]` |
| annotations | ENTITY(cat/e1), ACTION(sleep/a1), LOCATION(table/l1) |
| **canonical_hash** | `b8757b8732d3b2125ecb4af437d316d5f8fd5bc55a4adff64966b5e6ee5ef745` |
| **raw_hash** | `097cc017b1f21aae2d437a4ba382176c7bc95b57be3042d6ec06c5b7a546a4c9` |

### 03_ja_cat_run

| Field | Value |
|-------|-------|
| input | `猫が机の上で走っている。` |
| language | `ja` |
| tokens | `["猫が机の上で走っている"]` |
| annotations | ENTITY(cat/e1), ACTION(run/a2), LOCATION(table/l1) |
| **canonical_hash** | `b4d048bd6c10ea5114f4dd21ccfd9a839459a26e97d02aac5712f7a3d0db84ae` |
| **raw_hash** | `8c1ad4f2460c0e6a5e18e62d4fe24948b9d78c5be7a8a26c3c5e10af5c7829aa` |

### 04_en_neutral

| Field | Value |
|-------|-------|
| input | `the sky is blue` |
| language | `en` |
| tokens | `["the", "sky", "is", "blue"]` |
| annotations | _(none)_ |
| **canonical_hash** | `32b79f86d61fd3132bdffcec0fa13592b64dc175f9cfa3d54638f0af1da4872e` |
| **raw_hash** | `4d856725cba58f4435ccded2e23dc7842bfd7157f966d8164828f37740b3fb77` |

---

## Lock Summary

| ID | annotations | language | dual-hash | Status |
|----|-------------|----------|-----------|--------|
| 01_en_cat_sleep | PASS | en | **LOCKED** | ✅ |
| 02_ja_cat_sleep | PASS | ja | **LOCKED** | ✅ |
| 03_ja_cat_run | PASS | ja | **LOCKED** | ✅ |
| 04_en_neutral | PASS (empty) | en | **LOCKED** | ✅ |

**Result: 4 / 4 structure + dual-hash LOCKED**

Machine-readable: `PLP_R_GOLDEN_LOCK_v0_1.json`  
Reference implementation: `golden_lock_ref.py`

---

## Design Contracts (do not weaken)

1. Annotations are **Canonical Projection Candidates**, not semantic truth.
2. `MinimalProjector` is a research placeholder.
3. Dual hash is intentional:
   - `raw_hash` → integrity of source text
   - `canonical_hash` → integrity of projected state (header + canonical)
4. Changing serialization rules, header fields used in hash, or projector rules
   **requires a new golden version** (v0.1.3+), never silent mutation of these values.

---

## Cross-language acceptance

Any conforming implementation (Rust / Python / Go / …) MUST produce:

- identical `language`
- identical annotation multiset `(kind, value, key)`
- identical `canonical_hash`
- identical `raw_hash`

for every vector above under the fixed header parameters.

---

*実験は忠実に実際行って*

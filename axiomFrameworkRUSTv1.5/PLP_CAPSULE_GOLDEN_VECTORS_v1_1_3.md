# PLP Capsule Golden Vectors v1.1.3 — Cross-Language Report

**Date:** 2026-08-03  
**Rules:** PLP Capsule v1.1.3 deterministic serialization (hand-written, ryu-canonical floats, `chars()` escaping, `timestamp_ns` as string)  
**Rust:** local harness (`plp_capsule_v1_1_3.rs` rules)  
**Python:** minimal re-implementation of the same rules  

---

## Summary

| Case ID | Description | Rust Hash | Python Hash | Match |
|---------|-------------|-----------|-------------|-------|
| 01_empty | Empty observations + empty delta | `cc10ffc4…62a7e4` | `cc10ffc4…62a7e4` | **PASS** |
| 02_single_obs | Single observation (x=1.0, y=-2.5) | `a54b533a…34c3941e` | `a54b533a…34c3941e` | **PASS** |
| 03_two_obs | Two observations (geom + thermo) | `05188e6b…cc4dd655` | `05188e6b…cc4dd655` | **PASS** |
| 04_delta_added | DeltaKind=Added (two new keys) | `8ba444f0…1c5e543d` | `8ba444f0…1c5e543d` | **PASS** |
| 05_delta_modified | DeltaKind=Modified (Δx=+0.05) | `169e2140…508a0097` | `169e2140…508a0097` | **PASS** |
| 06_delta_remove_add | Removed + Added in same block | `86b6867b…8c415e05` | `86b6867b…8c415e05` | **PASS** |
| 07_obs_removed | Whole observation removed | `5c4433cd…425763be` | `5c4433cd…425763be` | **PASS** |
| 08_japanese | Japanese keys / ids / source | `70120de8…40237d35` | `70120de8…40237d35` | **PASS** |
| 09_control_chars | Tab + newline in source | `fa07d69c…1080c1ed` | `fa07d69c…1080c1ed` | **PASS** |
| 10_multi_mixed | Two obs + mixed delta | `3a0c75ab…fb8dfc0a` | `3a0c75ab…fb8dfc0a` | **PASS** |

**Result: 10 / 10 PASS (100%)**

---

## Full Hashes

```
01_empty             cc10ffc4ee1ac484366866b353b63b15ed5bda2052a24b11420e3ef06562a7e4
02_single_obs        a54b533ae4223cbef6d6227c957ac22d11efbcf61deead82bbc1e17134c3941e
03_two_obs           05188e6be25eac858002657f79b742faa35a3e499204dc5039e17209cc4dd655
04_delta_added       8ba444f0e17c3187fb3503966d70ccf424977d39cdddf94e462fbd1e1c5e543d
05_delta_modified    169e2140ff6de4b74626abc90f419f09987b9103878e747818f2b658508a0097
06_delta_remove_add  86b6867bd641106ae413528971dd7cfd0a46265529256e9ed5c28e698c415e05
07_obs_removed       5c4433cdabe88ff0dd5fea43014882a5845232d32ba83f37ad67cdc0425763be
08_japanese          70120de84fa3fe7daa9703588d8bb716514b82585d45e0841350a2cc40237d35
09_control_chars     fa07d69cb4e09585c488d4c7c29b2e8700470a6a3cf43fab643533e61080c1ed
10_multi_mixed       3a0c75ab4f22bf36fc541cc4280f992285aea8ac044ee557091a20f1fb8dfc0a
```

---

## Case Definitions

### Common Header (unless overridden)

| Field | Value |
|-------|--------|
| protocol | `PLP/1.1` |
| capsule_schema | `v1/capsule` |
| version | `1.1.3` |
| capsule_id | `00000000-0000-4000-8000-000000000001` |
| parent_id | `null` |
| clock | `42` |
| sequence | `7` |
| timestamp_ns | `"1700000000000000000"` (string) |
| source | `golden` (except 08, 09) |
| is_keyframe | `true` |
| hash_algorithm | `sha256` |

---

### 01_empty
- observations: `[]`
- delta: `{}`

### 02_single_obs
- 1 observation: `geom` / `v1/geometry` / `geometry` / `cam0`
  - `x=1.0`, `y=-2.5`
- delta: `{}`

### 03_two_obs
- observations (sorted by name):
  - `geom` / `v1/geometry` / `geometry` / `camera/front` → `radius=1.7`, `x=1.0`, `y=2.5`
  - `thermo` / `v1/thermal` / `thermal` / `sensor/0` → `energy=-0.1234`, `temp=0.0065`
- delta: `{}`

### 04_delta_added
- observation: `geom` / `cam` → `x=1.0`, `y=2.0`
- delta key: `geom.v1/geometry.geometry.cam`
  - kind: `added`
  - values: `x=Added(1.0)`, `y=Added(2.0)`

### 05_delta_modified
- observation: `geom` / `cam` → `x=1.05`, `y=2.0`
- delta key: `geom.v1/geometry.geometry.cam`
  - kind: `modified`
  - values: `x=Modified(0.05)`  *(difference, not absolute)*

### 06_delta_remove_add
- observation: `geom` / `cam` → `x=1.0`, `new=3.14`
- delta key: `geom.v1/geometry.geometry.cam`
  - kind: `modified`
  - values: `old=Removed`, `new=Added(3.14)`

### 07_obs_removed
- observations: `[]`
- delta key: `geom.v1/geometry.geometry.cam`
  - kind: `removed`
  - values: `x=Removed`

### 08_japanese
- source: `日本語テスト`
- observation: `形状` / `v1/geometry` / `geometry` / `カメラ/前`
  - `半径=1.7`, `x=1.0`
- delta: `{}`

### 09_control_chars
- source: `line1\tline2\nend`  (tab + newline, escaped as `\t` / `\n`)
- observation: `geom` / `cam0` → `x=0.5`
- delta: `{}`

### 10_multi_mixed
- observations:
  - `geom` / `cam0` → `x=0.5`, `y=-0.3`
  - `thermo` / `s0` → `temp=0.0065`
- delta key: `geom.v1/geometry.geometry.cam0`
  - kind: `modified`
  - values: `old_key=Removed`, `x=Modified(0.1)`, `z=Added(3.14)`

---

## Serialization Rules (locked)

1. Hand-written deterministic serializer (no language-default JSON).
2. Fixed field order: `header` → `observations` → `delta`.
3. Floats pre-canonicalized (ryu-style; `-0.0` → `"0"`).
4. `timestamp_ns` always a decimal **string**.
5. String escaping via Unicode scalar values (`chars()` / Python `for ch in s`).
6. Control characters (`< U+0020`) → `\u00XX`.
7. Map keys sorted (BTreeMap order).
8. `DeltaKind` / value kinds: stable strings `added` / `modified` / `removed`.

---

## Notes

- **Modified stores the difference** (`new − old`), not the absolute value.
- Stable observation key format used in delta:  
  `{name}.{schema}.{capability}.{observer_id}`
- Older repo vectors (`PLP/1.0` + `serde_json`) are a different generation and must not be mixed with these hashes.
- These 10 vectors are suitable as the official cross-language golden set for PLP Capsule v1.1.3.

---

## Status

**10/10 cases match between Rust and Python under identical rules.**  
Ready to be adopted as the reference Golden Vector suite for v1.1.3.

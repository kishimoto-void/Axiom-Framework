# AXIOM E2E Pipeline Report — PSS → PLP → Capsule → ACP → DCK

**Date**: 2026-08-08  
**Scope**: Design-level E2E + DCK v2.3 Dual-Hash classification  
**Baseline input**: `PythonでFizzBuzzを書いて`

---

## Pipeline

```
Input
  ↓
PSS          normalize / constraints / phase
  ↓
PLP          feature vector
  ↓
Capsule      deterministic hash  →  Hash A (physical)
  ↓
ACP          coordinate / transition
  ↓
DCK          evaluate_difference (numeric)
             DualHashEvaluation (kind)
```

## Key observations (numeric)

| Check | Result |
|-------|--------|
| `determinism_capsule_hash` | `True` |
| `determinism_acp_coordinate` | `True` |
| `determinism_dck_diff` | `True` |
| `baseline_diff` | `0.0` |
| `baseline_converged` | `True` |
| `noisy_diff_gt_baseline` | `True` |
| `heavy_diff_gt_baseline` | `True` |
| `ambiguous_flagged` | `True` |
| `contradiction_flagged` | `True` |
| `contradiction_diff` | `10.006397953309673` |
| `T02a_diff` | `0.6101349443226097` |
| `T02b_diff` | `0.0` |
| `T02_capsule_hash_match` | `False` |
| `T04_unique_hashes` | `3` |
| `T04_all_same` | `False` |

## Dual-hash vs baseline (T01)

| Case | diff | conv | dual_class | primary_kind | A same | B same | constraint |
|------|------|------|------------|--------------|--------|--------|------------|
| T01_baseline | 0.0000 | True | None | **None** | True | True | None |
| T02a_variation | 0.6101 | False | Compound | **Compound** | False | False | None |
| T02b_variation | 0.0000 | True | Compound | **Compound** | False | False | None |
| T03_noise | 2.5953 | False | Compound | **Compound** | False | False | None |
| T04a_order | 6.3302 | False | Compound | **Compound** | False | False | True |
| T04b_order | 6.3271 | False | Compound | **Compound** | False | False | True |
| T04c_order | 6.3347 | False | Compound | **Compound** | False | False | True |
| T05_constraint | 14.1704 | False | Compound | **Compound** | False | False | True |
| T06_ambiguous | 9.2326 | False | Compound | **Compound** | False | False | None |
| T07_contradiction | 10.0064 | False | Compound | **Constraint** | False | False | False |
| T08_heavy_noise | 2.7084 | False | Compound | **Compound** | False | False | None |
| T09a_synonym | 9.0580 | False | Compound | **Compound** | False | False | None |
| T09b_synonym | 9.0315 | False | Compound | **Compound** | False | False | None |
| T10_determinism_1 | 0.0000 | True | None | **None** | True | True | None |
| T10_determinism_2 | 0.0000 | True | None | **None** | True | True | None |

### Kind distribution

- **Compound**: 12
- **None**: 3
- **Constraint**: 1

## Interpretation

1. **Determinism**: T01 / T10_1 / T10_2 share the same Capsule hash and `diff=0` → Hash A/B identical → `None`.
2. **Noise / heavy noise**: Capsule A diverges from baseline → **Compound** (State + Semantic shift).
3. **Contradiction (T07)**: constraint_ok=False → **Constraint** primary kind (invalid interpretation path).
4. **Ambiguous (T06)**: phase=clarify, elevated residual; dual-hash captures identity change vs baseline.
5. **Order variants (T04*)**: close residuals (~6.33) but distinct capsule hashes → Compound — order still leaks into physical hash (PSS canonicalization target).
6. **Synonym (T09*)**: elevated residual + non-baseline A → synonyms not yet collapsed into same Hash A.

## Numeric residual table

| Case | input (trim) | phase | diff | rate |
|------|--------------|-------|------|------|
| T01_baseline | PythonでFizzBuzzを書いて | answer | 0.0000 | 1.0000 |
| T02a_variation | PythonでFizzBuzzのコードをお願い | answer | 0.6101 | 0.9695 |
| T02b_variation | FizzBuzzをPythonで実装してください | answer | 0.0000 | 1.0000 |
| T03_noise | えっと Pythonで 出来れば FizzBuzz お願い | answer | 2.5953 | 0.8702 |
| T04a_order | 初心者向けでPythonのFizzBuzz | confirm | 6.3302 | 0.6835 |
| T04b_order | Pythonで初心者向けFizzBuzz | confirm | 6.3271 | 0.6836 |
| T04c_order | FizzBuzzを初心者向けにPythonで | confirm | 6.3347 | 0.6833 |
| T05_constraint | 200文字以内 箇条書き禁止 Python コメント付き | confirm | 14.1704 | 0.2915 |
| T06_ambiguous | 簡単なのお願い | clarify | 9.2326 | 0.5384 |
| T07_contradiction | 100文字以内 詳しく説明 コードも全部書いて | clarify | 10.0064 | 0.4997 |
| T08_heavy_noise | Python Python Python お願い FizzBuzz | answer | 2.7084 | 0.8646 |
| T09a_synonym | 作成して | clarify | 9.0580 | 0.5471 |
| T09b_synonym | 実装して | clarify | 9.0315 | 0.5484 |
| T10_determinism_1 | PythonでFizzBuzzを書いて | answer | 0.0000 | 1.0000 |
| T10_determinism_2 | PythonでFizzBuzzを書いて | answer | 0.0000 | 1.0000 |

## Pass criteria (design-level)

| Criterion | Status |
|-----------|--------|
| Determinism (same input → same hash/diff) | **PASS** |
| Baseline converged (diff≈0) | **PASS** |
| Noise increases residual | **PASS** |
| Ambiguity flagged | **PASS** |
| Contradiction flagged | **PASS** |
| Dual-hash Constraint on T07 | **PASS** |

## Artifacts

- `golden/e2e/e2e_dualhash_rows.json`
- `golden/e2e/e2e_pipeline_pss_to_dck_results.json`
- `docs/DCK_DUAL_HASH_TAXONOMY_v2.3.md`

*E2E design-level run — AXIOM v1.6 / DCK v2.3*

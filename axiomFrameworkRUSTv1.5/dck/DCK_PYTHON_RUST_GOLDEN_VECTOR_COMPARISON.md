# DCK Python vs Rust Golden Vector Comparison Report

**Date**: 2026-08-04  
**Repository**: `kishimoto-void/Axiom-Framework`  
**Python DCK**: `src/modules/dck/` (v0.9.0)  
**Rust DCK**: `axiomFrameworkRUSTv1.5/dck/` (v2.0.0 modular + nalgebra)

---

## 1. Purpose

本レポートは、Python 実装（本家 v0.9）と Rust 実装（モジュール分割 + nalgebra 多次元対応版）の  
**論理的・数値的対応関係**をゴールデンベクタで検証し、互換性・差分・今後の整合方針を明確にするものである。

テスト方針は既存の PLP / LRP ゴールデンベクタスタイルに準拠する。

---

## 2. Scope of Comparison

| 項目 | Python v0.9 | Rust v2.0 | 比較対象 |
|------|-------------|-----------|----------|
| Config デフォルト | `DCKConfig` (Pydantic) | `DCKConfig` | ○ |
| StateEstimate + Mahalanobis | numpy + scipy Cholesky | nalgebra `DVector`/`DMatrix` + Cholesky | ○ |
| GapHistory (EMA velocity) | `utils.GapHistory` | `gap.GapHistory` | ○ |
| StubObserver / StubPredictor / StubExecutor | あり | あり | ○ |
| tick() 基本フロー (observe → forecast → gap → decision → lease → execute) | あり | あり | ○ |
| DecisionEngine / ResourceAllocator / Compensation | 本格実装 | 簡略化（閾値ベース） | △ 差分あり |
| DeterministicIDGenerator | あり | UUID ベース | △ |
| TwoTierEventCache / Snapshot | あり | 未実装 | × |
| 多次元共分散 | 対応 | nalgebra 本格対応 | ○ |

---

## 3. Shared Golden Vector Definition

### 3.1 Input Vector (共通)

```text
Config (shared defaults):
  max_gap_scale          = 100.0
  velocity_time_constant_tau = 2.0
  convergence_tolerance  = 2.0
  gap_history_size       = 10
  aging_factor           = 0.5
  weight_equivalence     = 1.0
  weight_velocity        = 1.5
  weight_risk            = 1.0
  cholesky_floor / psd_jitter ≈ 1e-8 ~ 1e-9

Intent:
  intent_id     = "intent_01"
  metric        = "temperature"
  target_value  = 25.0
  tolerance     = 0.5
  time_horizon  = 5
  base_priority = 1.0
  deadline      = None
  dependencies  = []

Telemetry (raw):
  temperature = 42.0

Turn = 1
```

### 3.2 Stub Behavior (意図的に揃えた部分)

| Step | Python Stub | Rust Stub | Notes |
|------|-------------|-----------|-------|
| Observe | mean=[42.0], var=0.25, conf=0.95 | mean=42.0, var=0.25, conf=0.95 | 一致 |
| Forecast decay | `max(0.5, 1.0 - 0.02*h)` = 0.9 | `(1.0 - 0.02*h).max(0.5)` = 0.9 | 一致 |
| Projected mean | 42.0 × 0.9 = **37.8** | 42.0 × 0.9 = **37.8** | 一致 |
| Projected variance | `0.25 * (1 + 0.1*h)` = **0.375** | `0.25 * (1 + 0.05*h)` = **0.3125** または固定 0.30 | **差分あり** |
| Confidence decay | `max(0.1, conf - 0.03*h)` | 同左 | 一致 |

### 3.3 Analytical Expected Values (Golden Numbers)

```text
proj_mean = 37.8

# 1D Mahalanobis: |proj_mean - target| / sqrt(var)
gap_python     = |37.8 - 25.0| / sqrt(0.375) ≈ 20.9023
gap_rust_scale = |37.8 - 25.0| / sqrt(0.3125) ≈ 22.8973
gap_rust_fixed = |37.8 - 25.0| / sqrt(0.30)   ≈ 23.3695

Decision thresholds (both):
  SafetyHalt  if gap > max_gap_scale * 1.5 = 150.0
  NoAction    if gap < convergence_tolerance * 0.001 ≈ 0.002
  else        ExecuteConvergence

→ 両実装とも gap ≈ 21~23 ≪ 150 かつ ≫ 0.002 のため
  expected action = EXECUTE_CONVERGENCE
  expected stage  = EXECUTED (StubExecutor always succeeds)
```

### 3.4 First-tick Velocity

初回 `GapHistory.push` は履歴が空のため **smoothed_velocity = 0.0**（両実装共通）。

---

## 4. Comparison Results

### 4.1 Numerical Golden Vector Table

| Metric | Python (expected) | Rust (expected) | Δ | Status |
|--------|-------------------|-----------------|---|--------|
| Observed mean | 42.0 | 42.0 | 0 | PASS |
| Projected mean | 37.8 | 37.8 | 0 | PASS |
| Equivalence gap | ≈ 20.902 | ≈ 22.90 ~ 23.37 | ~2.0 | **KNOWN DIFF** (var inflation formula) |
| Computed velocity (t=1) | 0.0 | 0.0 | 0 | PASS |
| Decision action | EXECUTE_CONVERGENCE | ExecuteConvergence | — | PASS (semantic) |
| Final stage | EXECUTED | Executed | — | PASS (semantic) |
| Lease reserve success | True (sufficient resources) | True | — | PASS |

### 4.2 Architectural / Behavioral Differences

1. **Predictor variance inflation**  
   - Python: `base_var * (1.0 + 0.1 * horizon)`  
   - Rust: `covariance * (1.0 + 0.05 * horizon)` または固定値  
   → gap に ~2 程度の差が出る。意図的に揃える場合は Rust 側を Python に合わせることを推奨。

2. **Decision path**  
   - Python: `DecisionEngine` + `DecisionContext` + 重み付きスコア + ResourceAllocator  
   - Rust: 単純な gap 閾値分岐（weights は計算のみで未使用に近い）  
   → 閾値付近の境界ケースで動作が分岐しうる。

3. **ID generation**  
   - Python: `DeterministicIDGenerator`（再現性重視）  
   - Rust: `Uuid::new_v4()`  
   → ゴールデンベクタの event_id / lease_id は文字列一致を求めない。

4. **Stage enum**  
   - Python: `INITIATED / PROJECTED / EXECUTED / CONVERGED / FAILED`  
   - Rust: `Projected / Executed / Failed`（簡略）  
   → 意味的対応は取れている。

5. **Locking / concurrency**  
   両者とも Semaphore + lock を使用。Rust は `join_all`、Python は `asyncio.gather`。意味的に同等。

6. **Resource / Lease**  
   基本の reserve → execute → commit_or_release の流れは対応。Python の方に Compensation / Snapshot が追加されている。

---

## 5. Conformance Verdict

| Category | Verdict | Comment |
|----------|---------|--------|
| Core numeric path (observe → project → gap → velocity) | **MOSTLY CONFORMANT** | variance formula のみ既知差分 |
| Decision outcome on this golden vector | **CONFORMANT** | 両方 EXECUTE_CONVERGENCE → EXECUTED |
| Resource / Lease basic flow | **CONFORMANT** | |
| Full feature parity (DecisionEngine, Cache, Snapshot, Compensation) | **NOT YET** | Rust は意図的にコアに絞っている |
| Multi-dimensional capability | **Rust ADVANTAGE** | nalgebra 本格対応 |

**総合**:  
共有ゴールデンベクタ（温度 42 → 目標 25, horizon=5）において、  
観測・予測平均・初回 velocity・最終アクション/ステージは一致する。  
gap の数値差は Predictor の分散増加係数の違いに起因する既知差分であり、  
「決定論的に同じ結果を返す」レベルにはまだ達していないが、  
**論理フローと主要な閾値判定は整合している**。

---

## 6. Recommended Next Steps

1. **Predictor variance を揃える**  
   Rust `StubPredictor` を Python と同じ `base_var * (1.0 + 0.1 * horizon)` に変更する。

2. **Deterministic ID / Clock を Rust にも導入**  
   ゴールデンベクタの完全再現性のため。

3. **境界ケースの追加ベクタ**  
   - gap がちょうど `max_gap_scale * 1.5` 付近  
   - gap が極小（NoAction）  
   - リソース不足（lease 失敗）  
   - 多次元 (temperature + humidity) ケース

4. **自動比較ハーネス**  
   同じ JSON 入力を両実装に与え、stage / action / gap（相対誤差許容）を assert する CI を検討。

---

## 7. Reproduction Notes

### Python (conceptual)
```python
# using StubObserver / StubPredictor / StubExecutor
# KernelBuilder → submit_intent → tick(1, {"temperature": 42.0})
# expect stage=EXECUTED, action=EXECUTE_CONVERGENCE, gap≈20.90
```

### Rust
```bash
cd axiomFrameworkRUSTv1.5/dck
cargo run
# expect Stage: Executed, Action: Some(ExecuteConvergence), Gap ≈ 22.9~23.4
```

---

## 8. Summary

- 共有ゴールデンベクタを定義し、主要数値を解析的に算出した。
- 観測・予測平均・velocity・アクション・ステージは一致。
- gap の差は StubPredictor の分散増加式の違いによる既知差分。
- アーキテクチャ上、Python が機能豊富、Rust が型安全・多次元・パフォーマンス寄り。
- 今後は分散式の統一と境界ベクタの追加で、より厳密な相互運用性を確保できる。

---

*Generated for Axiom-Framework / DCK cross-implementation conformance.*  
*Report location: `axiomFrameworkRUSTv1.5/dck/DCK_PYTHON_RUST_GOLDEN_VECTOR_COMPARISON.md`*

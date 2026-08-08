# PLP-R ↔ DCK Bridge — Research Note (Phase 2)

**Date**: 2026-08-09  
**Status**: Working demo (Python) + Rust skeleton  
**Depends on**: PLP-R v0.1.2 Dual Hash + DifferenceMetrics

---

## 目的

ProjectionCapsule の Dual Hash と Annotation 差分を、  
既存 DCK v2.3 の dual-hash taxonomy に接続する。

---

## マッピング（意図的）

| DCK v2.3 | PLP-R |
|----------|-------|
| HashA (Invariant / Physical) | `raw_hash` |
| HashB (Semantic / Interpretation) | `canonical_hash` |

分類行列:

| A same | B same | DualHashClass | 意味 |
|--------|--------|---------------|------|
| ✓ | ✓ | **None** | 同一 |
| ✓ | ✗ | **Semantic** | 同一入力・投影だけ違う |
| ✗ | ✓ | **State** | 入力変化・投影が偶然一致 |
| ✗ | ✗ | **Compound** | 両方違う |

Annotation の `DifferenceMetrics` は直交する定量情報として添付する。

---

## 実験結果（実測）

`python3 plp_dck_bridge_demo.py` の出力:

### 1. 同一入力
```
A = B = "cat sleeps on table"
→ [SAME] dual=None kind=None divergence=0.000 monitor=Continue
```

### 2. sleep vs run
```
A = "猫が机の上で寝ている。"
B = "猫が机の上で走っている。"
→ [DIVERGE] dual=Compound kind=Compound divergence=0.500 monitor=AskUser
  ACTION: sleep removed / run added
  Monitor: "Canonical state candidates diverged: ACTION=mixed"
  candidates: [PlanSleep, PlanRun]
```

### 3. 中立 vs cat
```
→ [DIVERGE] dual=Compound divergence=1.000 monitor=AskUser
  ENTITY/LOCATION/ACTION all added
```

**結論**: Dual Hash 分類 + Annotation 差分 + Monitor が一本のパイプラインで動作した。

---

## ファイル

| ファイル | 役割 |
|---------|------|
| `plp_dck_bridge_v0_1.rs` | Rust 側ブリッジ（DCK taxonomy 準拠） |
| `plp_dck_bridge_demo.py` | Python 実働デモ |
| `PLP_DCK_BRIDGE_DEMO_RESULT.json` | 実測結果 |

---

## 次（Phase 3 へ）

Monitor の AskUser を「ユーザー確認待ち」状態機械として最小デモ化する。

```
Capsule A / B
    → compare_capsules
    → AskUser
    → (user picks PlanSleep | PlanRun)
    → adopted CanonicalState を次ターンの baseline に
```

---

*実験は忠実に実際行って*

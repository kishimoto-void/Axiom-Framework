# PLP State Projection — Research Note (PLP-R)

**Research revision**: v0.1.2 (SuperGrok refinement 2026-08-09)  
**Payload version** (serialized, Golden-locked): `0.1.1`  
**Status**: Research Prototype — Phase 1 hashes LOCKED  
**Production**: PLP Capsule v1.1.3（並列・非破壊）

---

## 位置付け

```
Production
  PLP Capsule v1.1.3          ← 安定系（壊さない）

Research  PLP-R
  v0.1.1  3層 + Dual Hash + Golden 構造
  v0.1.2  Golden hash LOCK + Projector 階層
          + DifferenceMetrics + Monitor skeleton
```

昇格パス:

```
PLP-R v0.1 → v0.2 → … → Candidate → PLP v2.0
```

---

## 設計契約（不変）

1. **PLP は意味解析をしない**  
   責務は「入力 → 決定論的 Canonical State への投影」のみ。

2. **Annotation = Canonical Projection Candidate**  
   Semantic Truth ではない。  
   MinimalProjector の ENTITY/ACTION/LOCATION は研究用プレースホルダ。

3. **3層構造**

   | 層 | 役割 |
   |----|------|
   | Raw Text | LLM が読む原文 |
   | Canonical State | エージェント / DCK が共有・比較 |
   | Dual Hash | 完全性 |

4. **Dual Hash（意図的判断）**

   ```
   raw_hash        = SHA256(raw_text)              → 原文の完全性
   canonical_hash  = SHA256(header + Canonical)    → 状態表現の完全性
   ```

   Raw を canonical_hash に入れない理由:  
   状態同一性を原文の表現ゆれから独立させるため。RFC に明記予定。

---

## Projector 階層（v0.1.2）

| Projector | 出力 | 用途 |
|-----------|------|------|
| **TokenOnlyProjector** | language + tokens のみ（annotations 空） | 意味を一切主張しない baseline |
| **MinimalProjector** | 上記 + 仮ルール annotations | Golden / デモ用プレースホルダ |

`annotation_status` meta:
- TokenOnly → `"none"`
- Minimal → `"canonical_projection_candidate"`

---

## Phase 進捗

| Phase | 内容 | 状態 |
|-------|------|------|
| **1** | Golden Vector（クロス言語決定論） | **✅ hashes LOCKED** |
| **2** | DCK 接続 | **✅ Bridge demo PASS** (DualHashClass + Metrics + Monitor) |
| **3** | Monitor | **✅ Demo PASS (15/15)** |
| **4** | より豊かな決定論的 Projector | 未着手 |

### Phase 1 — Golden（LOCKED）

Python reference (`golden_lock_ref.py`) と Rust が同一ルールで:

| ID | canonical_hash | raw_hash |
|----|----------------|----------|
| 01_en_cat_sleep | `b130e1ff…abe629b` | `5dd6fc5d…063c039` |
| 02_ja_cat_sleep | `b8757b87…e5ef745` | `097cc017…546a4c9` |
| 03_ja_cat_run | `b4d048bd…0db84ae` | `8c1ad4f2…c7829aa` |
| 04_en_neutral | `32b79f86…1da4872e` | `4d856725…0b3fb77` |

詳細: `PLP_R_GOLDEN_LOCK_v0_1.json` / `PLP_R_GOLDEN_VECTORS_v0_1.md`

### Phase 2 — DCK bridge

```rust
let diff = diff_canonical(&a, &b);
let metrics = DifferenceMetrics::from_diff(&a, &b, &diff);
// metrics.overlap_ratio / divergence / kind_status
// → 既存 DCK 数値評価へ渡す
```

### Phase 3 — Monitor

```rust
let decision = monitor_decide(&metrics, integrity_ok, "PlanA", "PlanB");
// Continue | AskUser { summary, candidates } | Abort { reason }
```

---

## 使い方

```rust
// Demo placeholder (Golden 用)
let b = ProjectionBuilder::with_minimal();
let cap = b.project("猫が机の上で寝ている。")?;

// Honest baseline（意味を一切出さない）
let b2 = ProjectionBuilder::with_token_only();
let pure = b2.project("猫が机の上で寝ている。")?;

// Golden
for gv in official_golden_vectors() {
    check_golden(&b, &gv)?;
}

// DCK + Monitor
let diff = diff_canonical(cap.canonical_state(), other.canonical_state());
let m = DifferenceMetrics::from_diff(cap.canonical_state(), other.canonical_state(), &diff);
let d = monitor_decide(&m, true, "AgentA", "AgentB");
```

---

## 次の実験

1. DifferenceMetrics → 既存 DCK モジュールへの実接続
2. Monitor の最小マルチエージェントデモ（2 Plan → Diff → AskUser）
3. Go 実装で Golden クロス言語一致の第三確認

---

*実験は忠実に実際行って*

# AXIOM Framework v2.0 — Priority & Design Contract

**Date**: 2026-08-09  
**Status**: Design lock (research → production path)  
**Based on**: PLP-R Phase 1–3 results + Framework prioritization

> 実験は忠実に実際行って

---

## 1. 認識の確認

現状の進捗から見た **v2.0 の優先順位は正しい**。

```
最優先     ACP v2.0  +  Capsule v2.0
その後     Framework API 固定
           Golden Vector v2
           Conformance Test
完成条件   PSS→PLP→Capsule→ACP→DCK が言語非依存で決定論的
```

PLP-R（State Projection）を正式採用するなら、  
Capsule / ACP は「データコンテナ」ではなく **Projection を証明可能な基盤** として設計する。

---

## 2. v2.0 優先順位（確定）

### Priority 1 — ACP v2.0

| 項目 | 内容 | PLP-R からの接続 |
|------|------|------------------|
| ハッシュ仕様の整理 | 決定論的シリアライズ + アルゴリズム固定 | Dual Hash 規則を継承 |
| Domain Separation の強化 | 用途別 domain tag（raw / canonical / proof / constraint） | HashA / HashB の分離と整合 |
| Proof Chain の最適化 | header → state → proof の一方向鎖 | Capsule integrity → ACP seal |
| State Projection (R) 対応 | Canonical State を第一級の証明対象に | PLP-R Capsule 3層を直接入力可 |
| バージョン互換性管理 | payload version ≠ crate version | Golden ロック経験を反映 |

**ACP の責務（v2.0）**

```
ACP seals what is true at a point in time.

  HashA (Invariant)  = 原文 / 物理入力の完全性   ← raw_hash
  HashB (Projected)  = 投影状態の完全性         ← canonical_hash
  Proof              = A と B の関係 + 制約判定
```

Domain Separation 例:

```
axiom:v2:raw:sha256:…
axiom:v2:canonical:sha256:…
axiom:v2:proof:sha256:…
axiom:v2:constraint:…
```

### Priority 2 — Capsule v2.0

| 項目 | 内容 | PLP-R からの接続 |
|------|------|------------------|
| ACP との完全統合 | Capsule 生成時に ACP seal を必須化 | ProjectionCapsule → sealed Capsule |
| Header の整理 | protocol / version / id / clock / domain | ProjectionHeader を整理して昇格 |
| **A(不変) / B(可変) 境界の明確化** | A = Raw + raw_hash / B = Canonical + canonical_hash | Dual Hash そのもの |
| Projection 情報の保持 | annotations は Candidate として保持 | annotation_status 契約を維持 |
| 差分生成の高速化 | Canonical のみ比較（Raw を混ぜない） | diff_canonical + DifferenceMetrics |

**Capsule の責務（v2.0）**

```
Capsule =
  A-layer  (Immutable)   Raw Text + raw_hash
  B-layer  (Projected)   CanonicalState + canonical_hash
  Seal     (ACP)         proof over (A, B, header)
```

LLM は Raw を読む。  
Agent / DCK / Monitor は Canonical だけを比較する。  
ACP は両方の完全性と関係を証明する。

### Priority 3 — Framework API 固定

```
PSS → PLP → Capsule → ACP → DCK
```

各境界の I/O 型を **凍結**する。

| 境界 | Input | Output |
|------|-------|--------|
| PSS → PLP | NormalizedInput | RawText + language hint |
| PLP → Capsule | RawText | ProjectionCapsule (A+B+hashes) |
| Capsule → ACP | ProjectionCapsule | SealedCapsule (with proof) |
| ACP → DCK | SealedCapsule × 2 | DualHashEvaluation + DifferenceMetrics |
| DCK → Monitor | Evaluation + Metrics | MonitorDecision |

### Priority 4 — Golden Vector v2

- 言語横断（Python / Rust / Go）で **同一 hash・同一 annotations**
- ACP seal を含む end-to-end vector
- PLP-R v0.1.2 の 4 Golden を母体に拡張

### Priority 5 — Conformance Test

他実装が同じ結果を返すことを保証するテストスイート。

---

## 3. v2.0 完成条件（Deterministic Pipeline）

```
Input
  ↓
PSS
  ↓
PLP   (State Projection — meaning を主張しない)
  ↓
Capsule  (A immutable / B projected)
  ↓
ACP   (seal + domain-separated hashes)
  ↓
DCK   (DualHash class + annotation metrics)
  ↓
Output  (Continue | AskUser | Abort)
```

**完成の定義**

1. 同一 Input → 全言語で同一 Capsule hashes
2. 同一 2 Capsules → 同一 DualHashClass + DifferenceMetrics
3. 同一 Metrics → 同一 MonitorDecision
4. Conformance suite が CI で green

---

## 4. PLP-R 成果の昇格マップ

| PLP-R (Research) | v2.0 Production への行き先 |
|------------------|---------------------------|
| Dual Hash (raw / canonical) | Capsule A/B + ACP HashA/HashB |
| Annotation = Candidate | Capsule B-layer 契約（Semantic Truth 禁止） |
| DifferenceMetrics | DCK 数値入力 |
| MonitorDecision | Runtime / multi-agent 制御 |
| Golden v0.1.2 (4 vectors) | Golden Vector v2 の母体 |
| TokenOnly / MinimalProjector | PLP v2 Projector 階層の起点 |

昇格パス:

```
PLP-R v0.1.2  →  Capsule v2.0 draft  →  ACP v2.0 draft
              →  Golden v2  →  Conformance
              →  Candidate  →  PLP v2.0 / Framework v2.0
```

---

## 5. 設計上の不変契約（v2.0 で壊さない）

1. **PLP は意味解析をしない**  
   Annotation は Canonical Projection Candidate。

2. **A と B は混ぜない**  
   Raw を canonical_hash に折り込まない。  
   Canonical だけを DCK が比較する。

3. **Domain Separation 必須**  
   同じバイト列でも domain tag が違えば別 hash 空間。

4. **Production を研究で壊さない**  
   PLP Capsule v1.1.3 は安定系のまま。  
   v2.0 は並列線 → Candidate → 昇格。

5. **Header 由来ノイズで Monitor を動かさない**  
   Monitor の主信号は Canonical State 差分。

---

## 6. 実装ロードマップ（推奨順序）

### Step A — ACP v2.0 RFC（仕様先行）

- [ ] Hash domain tags の正式定義
- [ ] Proof chain フォーマット（header + A + B）
- [ ] Version / compatibility matrix
- [ ] State Projection 入力スキーマ

### Step B — Capsule v2.0 draft

- [ ] A/B 境界を型で強制
- [ ] Header 正規化（hash 対象フィールドの凍結）
- [ ] ACP seal 必須パス
- [ ] diff は Canonical only（高速パス）

### Step C — Pipeline I/O freeze

- [ ] 各 Stage の型を単一の `types/v2` に集約
- [ ] Python / Rust で同一スキーマ生成

### Step D — Golden Vector v2 + Conformance

- [ ] PLP-R Golden を ACP seal 付きに拡張
- [ ] Cross-language CI
- [ ] Monitor シナリオ（15 checks）を conformance に編入

### Step E — Runtime 接続

- [ ] UPR Stage として PLP → Capsule → ACP → DCK を正式配線
- [ ] multi-agent AskUser デモを Runtime 上で再現

---

## 7. なぜこの順か

- **ACP / Capsule が先**: ハッシュと境界が固まらないと Golden v2 も Conformance も空中戦になる。
- **PLP-R を後回しにしない**: すでに Dual Hash・Candidate 契約・Monitor が実測済み。これを ACP/Capsule の土台に使うのが最短。
- **API 固定は土台の後**: 動く決定論パイプラインが先。型凍結はそのスナップショット。
- **LLM Runtime / 検証機構は自然な延長**: Projection が証明可能になった時点で、検証は ACP の上に載る。

---

## 8. 参照（既存成果）

| 成果 | 場所 |
|------|------|
| PLP-R Phase 1–3 結果 | `docs/plp-r/` |
| Monitor 15/15 PASS | `docs/plp-r/PLP_MONITOR_DEMO_TEST_REPORT.md` |
| Golden LOCKED | `docs/plp-r/PLP_R_GOLDEN_LOCK_v0_1.json` |
| DualHash マッピング | `docs/plp-r/RESEARCH_PLP_DCK_BRIDGE_v0_1.md` |

---

*実験は忠実に実際行って*

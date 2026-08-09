# Round Consensus Protocol — Concept Draft v0.1

**Date**: 2026-08-09  
**Status**: Concept Draft (AXIOM Framework v2.0)  
**Depends on**: ACP v2.0 / Capsule v2.0 / PLP-R / DCK / Monitor  
**Layer**: Multi-agent coordination above Protocol Constitution (ACP)

> 実験は忠実に実際行って

---

## 1. 背景

従来のマルチ LLM は、多くの場合「複数の回答を生成し、多数決または統合する」構成を採る。

課題:

- 全モデルが同じ方向に誤る可能性
- 高性能モデルへの依存
- 個々のモデルの専門性が十分に活かされない

本プロトコルでは「全員が同じことをする」のではなく、**役割を分離し、役割を循環させる**ことを基本思想とする。

---

## 2. 基本思想

各エージェントは固定人格ではなく、**ラウンドごとに役割を交代**する。

### 役割（列挙可能）

```text
Role = Observer | Reasoner
```

将来拡張を妨げないよう enum とする。現時点の最小構成は:

- **Observer** × 1
- **Reasoner** × N（典型は 2）

### Clock Rotation 例（3 エージェント α, β, γ）

| Round | Observer | Reasoner | Reasoner |
|-------|----------|----------|----------|
| 0 | α | β | γ |
| 1 | β | γ | α |
| 2 | γ | α | β |
| 3 | α | β | γ |

Observer 権限が特定モデルに固定されることを防ぐ。

---

## 3. HashA / HashB

既存の Capsule A/B・ACP Dual Hash・PLP-R Dual Hash と整合させる。

### HashA — 不変契約（Invariant）

ラウンド中は変更しない。監視対象。

内容例:

- User Goal
- Axiom
- Safety Rules
- Framework Constraints
- Immutable Policy

ACP / Capsule では **A-layer / HashA (Invariant)** に対応する。

### HashB — 推論結果（Projected）

毎ラウンド生成される候補。

内容例:

- Candidate
- Projection（PLP Canonical State を含みうる）
- Improvement
- Proposal
- Discussion

ACP / Capsule では **B-layer / HashB (Projected)** に対応する。

---

## 4. Observer の責務

Observer は**推論しない**。回答品質も評価しない。

責務は契約遵守のみ:

- HashA 違反の検出
- 先回り・推測の検出
- Goal 逸脱の検出
- Hallucination の疑い
- 必要に応じたユーザー確認

### ObserverVerdict（型固定）

```text
ObserverVerdict =
    Accept
  | Revise { reason: string }
  | AskUser { summary: string, candidates: string[] }
  | Abort { reason: string }
```

既存 `MonitorDecision`（Continue / AskUser / Abort）と整合する。  
`Revise` は Reasoner への差し戻し用に追加した拡張。

---

## 5. Round Consensus 手順

1. Reasoner が HashB 候補を生成（複数可）
2. Observer が HashA との整合性を確認 → `ObserverVerdict`
3. `Revise` → Reasoner 再生成 / `AskUser` → ユーザー確認 / `Abort` → 停止
4. `Accept` → **Seal** 生成
5. 役割を Clock Rotation
6. Round Reset

### Seal 対象（固定）

```text
Seal = ACP.proof(
  HashA,
  approved_HashB,
  round,
  role_assignment
)
```

「このラウンドで、この HashA の下で、この HashB が Observer により契約適合と判定された」ことの証明。  
Capsule / ACP v2.0 Proof Chain に直結する。

---

## 6. Round Reset

各ラウンド終了時、全エージェントの推論状態をリセットする。

### 保持してよい情報

- HashA
- 承認済み HashB（およびその Seal）
- ACP Seal 履歴（必要最小限）
- Round 番号
- 現在の role_assignment（次ラウンド開始用）

### 持ち越さない情報

- 途中の思考過程
- 内部 hidden state
- 未承認の HashB 候補
- Observer 以外が保持していた一時メモ

---

## 7. 専門性の活用

ベンチマーク順位ではなく、各モデルの専門性を活かす。

例:

- 数学特化
- コード生成特化
- 創造性特化
- メタ認知特化
- 長文要約特化

Reasoner では尖った能力を発揮し、Observer は HashA 整合のみを見る。  
異端的・創造的な発想を維持しつつ、不変契約による安全性を確保する。

---

## 8. 差異収束（Difference Convergence）— 設計仮説

Round Reset の目的は思考履歴の蓄積ではない。

承認済みの HashA と HashB のみを引き継ぐことで:

- ラウンドごとに推論を再評価できる
- 誤った推論の累積を抑制できる
- エージェント間の差異を HashA 基準で測り、収束性を**検証可能**にする

### 主張の境界（重要）

> HashA という共通の不変基準を維持しつつ、各ラウンドで内部状態をリセットすることで、不要なバイアスの累積を抑え、**差異の収束性を高められる可能性がある**。

**保証ではない。** 収束性は DCK の数値（divergence / overlap / DualHashClass）で実験的に検証する設計仮説である。

### 測定との接続

| 測定 | 手段 |
|------|------|
| ラウンド間 HashB 差 | `diff_canonical` + `DifferenceMetrics` |
| Dual-hash 分類 | `DualHashEvaluation`（None / Semantic / State / Compound） |
| Observer 介入率 | Accept / Revise / AskUser / Abort の頻度 |
| 専門性の重なり | annotation overlap（kind_status） |

---

## 9. Design Goal

Round Consensus Protocol の目的は:

> 「最も優秀な単一 LLM を決めること」

ではない。

目的は:

> **専門性を持つ複数の LLM が、不変契約（HashA）の下で協調し、安全かつ創造的な推論を継続できるプロトコルを提供すること**

ACP は、その協調を保証するための **Protocol Constitution** として機能する。

---

## 10. 既存 AXIOM 部品との対応

| Round Consensus | AXIOM 既存 |
|-----------------|------------|
| HashA | Capsule A-layer / ACP HashA / 不変契約 |
| HashB | Capsule B-layer / PLP Canonical Projection / ACP HashB |
| Observer | Monitor（契約監視） |
| Reasoner | PLP Projection + LLM Agent |
| Seal | ACP v2.0 proof |
| 差異測定 | DCK DualHash + DifferenceMetrics |
| Round Reset | Seal 済みスナップショットのみ保持 |
| ObserverVerdict | MonitorDecision の拡張 |

PLP-R（State Projection）は Reasoner が HashB を生成する際の**決定論的投影層**として載る。  
意味解析は LLM 側、PLP は Canonical State への投影のみ、という契約を維持する。

---

## 11. 最小デモ（実装パス）

1. 固定 HashA（Goal + Safety テキスト）
2. 3 エージェント相当のスタブ（または 2 Reasoner + 1 Observer）
3. Reasoner → HashB（PLP-R MinimalProjector でも可）
4. Observer → ObserverVerdict
5. Accept 時のみ ACP 風 Seal（dual hash で代用可）
6. Reset → 次ラウンド
7. DCK で round 間 divergence を記録

成功条件（デモ）:

- Observer が品質ではなく契約のみを見る
- Seal 後に持ち越す状態が (HashA, approved HashB, Seal, round, roles) に限定される
- divergence がログに残る（縮小の有無は問わない）

---

## 12. 非目標（Non-goals）v0.1

- 多数決による最終回答決定
- Observer による「より良い回答」の選定
- 差異の単調減少の保証
- 特定 LLM ベンダーへの依存

---

## 13. 次の成果物

- [ ] `ObserverVerdict` / `Seal` の型を `types/v2` に追加
- [ ] 3 エージェント・2 ラウンドの最小デモ（Python）
- [ ] DCK による round 間 divergence ログ
- [ ] ACP v2.0 RFC との Seal ペイロード整合
- [ ] Golden シナリオ（契約違反検出・AskUser・Accept）

---

## 参照

| 文書 | 場所 |
|------|------|
| v2.0 優先順位 | `docs/AXIOM_V2_PRIORITY_AND_DESIGN.md` |
| ACP / Capsule sketch | `docs/AXIOM_ACP_CAPSULE_V2_SKETCH.md` |
| PLP-R 結果 | `docs/plp-r/` |
| Monitor 15/15 | `docs/plp-r/PLP_MONITOR_DEMO_TEST_REPORT.md` |
| DualHash マッピング | `docs/plp-r/RESEARCH_PLP_DCK_BRIDGE_v0_1.md` |

---

*実験は忠実に実際行って*

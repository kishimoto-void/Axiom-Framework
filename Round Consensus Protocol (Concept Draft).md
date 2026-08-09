AXIOM Framework v2.0

Round Consensus Protocol (Concept Draft)

1. 背景

従来のマルチLLMは、多くの場合「複数の回答を生成し、多数決または統合する」という構成を採用している。

しかし、この方式では、

- 全モデルが同じ方向に誤る可能性
- 高性能モデルへの依存
- 個々のモデルの専門性が十分に活かされない

といった課題がある。

本プロトコルでは、「全員が同じことをする」のではなく、「役割を分離し、役割を循環させる」ことを基本思想とする。

---

2. 基本思想

各エージェントは固定された人格ではなく、ラウンドごとに役割を交代する。

役割は以下の3種類で構成される。

- Observer（監視）
- Reasoner（推論）
- Reasoner（推論）

例：

Round 0

- α：Observer
- β：Reasoner
- γ：Reasoner

Round 1

- β：Observer
- α：Reasoner
- γ：Reasoner

Round 2

- γ：Observer
- α：Reasoner
- β：Reasoner

このローテーションを繰り返す。

---

3. HashA / HashB

HashA

HashAは不変契約である。

内容例：

- User Goal
- Axiom
- Safety Rules
- Framework Constraints
- Immutable Policy

HashAは監視対象であり、ラウンド中は変更されない。

---

HashB

HashBは推論結果である。

内容例：

- Candidate
- Projection
- Improvement
- Proposal
- Discussion

HashBは毎ラウンド生成される。

---

4. Observerの責務

Observerは推論を行わない。

責務は以下に限定する。

- HashA違反の検出
- 先回り・推測の検出
- Goal逸脱の検出
- Hallucinationの疑い
- 必要に応じたユーザー確認

Observerは回答品質ではなく、契約遵守のみを評価する。

---

5. Round Consensus

1. ReasonerがHashB候補を生成
2. ObserverがHashAとの整合性を確認
3. 問題があれば修正またはユーザーへ確認
4. 問題がなければSealを生成
5. 役割をローテーション

---

6. Round Reset

各ラウンド終了時に、全エージェントは推論状態をリセットする。

保持する情報は以下のみとする。

- HashA
- 承認済みHashB
- ACP Seal
- Round番号

途中の思考過程や内部状態は次ラウンドへ持ち越さない。

---

7. Clock Rotation

役割は時計回り（Clock Rotation）に循環する。

Round 0

α → Observer

↓

Round 1

β → Observer

↓

Round 2

γ → Observer

↓

Round 3

α → Observer

...

これにより、特定のモデルへ監視権限が固定されることを防ぐ。

---

8. 専門性の活用

本プロトコルはベンチマーク順位ではなく、各モデルの専門性を活かすことを目的とする。

例：

- 数学特化
- コード生成特化
- 創造性特化
- メタ認知特化
- 長文要約特化

Reasonerでは尖った能力を最大限発揮し、ObserverがHashAとの整合性のみを確認する。

これにより、異端的・創造的な発想を維持しながら、不変契約による安全性を確保できる。

---

9. 差異収束（Difference Convergence）

Round Resetの目的は、思考履歴を蓄積し続けることではない。

各ラウンド終了時に推論状態を初期化し、承認済みのHashAとHashBのみを次ラウンドへ引き継ぐことで、不要なバイアスや思考の偏りを持ち越さない。

その結果、

- ラウンドごとに推論を再評価できる
- 誤った推論の累積を抑制できる
- エージェント間の差異をHashAを基準として徐々に収束させられる可能性がある

ただし、差異が必ず縮小することを保証するものではなく、その収束性は実験・評価によって検証する設計仮説である。

---

10. Design Goal

Round Consensus Protocolの目的は、

「最も優秀な単一LLMを決めること」

ではない。

目的は、

「専門性を持つ複数のLLMが、不変契約（HashA）の下で協調し、安全かつ創造的な推論を継続できるプロトコルを提供すること」

である。

ACPは、その協調を保証するためのProtocol Constitutionとして機能する。

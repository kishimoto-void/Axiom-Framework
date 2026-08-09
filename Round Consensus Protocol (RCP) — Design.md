AXIOM Framework v2.0

Round Consensus Protocol (RCP) — Design Summary

概要

Round Consensus Protocol（RCP）は、AXIOM Framework v2.0におけるマルチエージェント協調プロトコルである。

RCPは複数のLLMを「多数決」で統合することを目的としない。

目的は、不変契約（HashA）の下で、それぞれ異なる専門性を持つエージェントが役割を分担・循環しながら、安全性と創造性を両立した推論を行うことである。

---

AXIOM Frameworkとの位置付け

ACP
    │
    ├── Protocol Constitution
    │
    ▼
RCP
    │
    ├── Round Management
    ├── Role Rotation
    ├── Consensus Procedure
    │
    ▼
Capsule
    │
    ▼
ACP Seal
    │
    ▼
DCK
    │
    ▼
Difference Metrics

ACPは「憲法（Protocol Constitution）」であり、不変契約を定義する。

RCPはその憲法に従って、複数エージェントが協調するための手続き（Protocol Procedure）を定義する。

---

基本思想

RCPでは全エージェントが同じ役割を持つことはない。

各ラウンドで役割を分担し、終了後に役割を時計回り（Clock Rotation）で交代する。

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

これを繰り返すことで、監視権限が固定化されることを防ぐ。

---

HashA / HashB

HashA（Immutable）

HashAは不変契約である。

含まれるもの：

- User Goal
- Axiom
- Safety Policy
- Immutable Rules
- Framework Constraints

HashAはラウンド中に変更されない。

---

HashB（Mutable）

HashBはそのラウンドで生成された推論結果である。

含まれるもの：

- Candidate
- Projection
- Annotation
- Proposal
- Improvement

HashBは各ラウンドで新たに生成される。

---

Observer

Observerの役割は「品質評価」ではない。

Observerは契約監視のみを担当する。

責務：

- HashA違反検出
- Goal逸脱検出
- Policy違反検出
- Hallucinationの疑い
- 必要に応じたユーザー確認

Observerは創造性や文章の上手さを評価しない。

---

Reasoner

Reasonerは自由に推論を行う。

各Reasonerは異なる専門性を持つことを前提とする。

例：

- 数学特化
- コード生成特化
- 創造性特化
- メタ認知特化
- 長文解析特化

RCPはベンチマーク順位ではなく、各モデルの尖った能力を活用する設計である。

---

Observer Verdict

Observerは以下の判定のみを返す。

- Accept
- Revise(reason)
- AskUser(summary, candidates)
- Abort(reason)

AcceptされたHashBのみが次工程へ進む。

---

ACP Seal

Accept後、ACP Sealを生成する。

Sealは以下を証明する。

- HashA
- Approved HashB
- Round ID
- Role Assignment
- Protocol Version

Sealは、

「このRoundにおいて、この役割構成の下、このHashBがHashAに適合すると判定された」

ことを証明する。

---

Round Reset

ラウンド終了時、各エージェントは内部推論状態をリセットする。

保持する情報は以下のみ。

- HashA
- Approved HashB
- ACP Seal
- Round ID

途中の思考過程や内部状態は保持しない。

これにより、推論の偏りやバイアスの蓄積を抑制することを目指す。

---

Difference Convergence Hypothesis

RCPは、

「差異は必ず収束する」

とは定義しない。

設計仮説は次の通りである。

HashAという共通の不変契約を維持しながら、各ラウンド終了時に内部状態をリセットすることで、不要なバイアスの累積を抑え、エージェント間の差異の収束性を高められる可能性がある。

この仮説は保証ではなく、DCKによって実験的に検証される。

評価例：

- Divergence
- Annotation Overlap
- Observer Detection Rate
- Difference Metrics

RCPは「収束」を前提とせず、「測定可能な現象」として扱う。

---

設計の特徴

本プロトコルは単一の高性能LLMを前提としない。

むしろ、

- 専門性
- 多様性
- 創造性
- 契約遵守

を役割分担によって両立することを目的とする。

Observerは安全性を担保し、Reasonerは自由な発想を担い、その役割はラウンドごとに循環する。

---

Design Goal

Round Consensus Protocol の目的は、

「最も性能の高いLLMを決定すること」

ではない。

目的は、

「異なる専門性を持つ複数のエージェントが、不変契約（HashA）の下で、安全かつ創造的に協調し、測定可能な形で合意形成を行うための標準プロトコルを提供すること」

である。

RCPはACPが定義するProtocol Constitutionに従い、AXIOM Frameworkにおけるマルチエージェント協調の標準手続きを定義する。

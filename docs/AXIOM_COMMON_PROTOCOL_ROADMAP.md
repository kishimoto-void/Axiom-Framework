# AXIOM COMMON PROTOCOL Roadmap

> 今の段階なら、私は**「実装を増やす」より「標準を育てる」フェーズ**に入ることをおすすめします。

---

## Phase 1 — Reference Standard（現在〜1.1）

**目標：仕様を固定する**

### 成果物

- ✅ `common_protocol.py`
- README
- SPECIFICATION.md（RFC形式）
- Golden Test Vectors
- Security Considerations
- ChangeLog

### 完了条件

- Python実装を「Reference」と宣言
- 後方互換ポリシー策定
- Semantic Versioning採用

---

## Phase 2 — Multi-language Validation（v1.1〜1.2）

**目標：「Pythonだけ動く」を卒業する。**

### 実装例

- Rust
- Go
- C#
- Java
- TypeScript

各実装で

```
同じJSON
  ↓
同じ Coordinate ID
  ↓
同じ Genesis Hash
  ↓
同じ Core Hash
```

になることを確認します。

これができると **言語非依存プロトコル** として非常に強くなります。

---

## Phase 3 — RFC Document

ここはコードではありません。

例えば

**RFC-AXIOM-0001**

- Abstract
- Terminology
- Canonical JSON
- Coordinate ID
- Genesis
- Transition
- Merge
- Lamport Clock
- Proof
- Extension
- Security Considerations
- IANA Considerations
- References

この形です。

コードより重要になる段階です。

---

## Phase 4 — Capsule Standard

ここでようやく **Capsule** を正式仕様化します。

役割は

```
AXIOM
  ↓
共通骨格
────────────
Capsule
  ↓
可変情報
```

例えば

- Observer
- Delta
- Reasoning
- Embedding
- Memory
- Execution Result

などです。

AXIOMは触らず、**Capsuleだけ拡張**できます。

---

## Phase 5 — LRP Integration

ここで

```
LRP
  ↓
AXIOM
  ↓
Capsule
```

になります。

つまり

```
Reasoning
  ↓
Transition生成
  ↓
AXIOMへ保存
```

という流れです。

---

## Phase 6 — PLP Integration

さらに **PLP** との接続。

```
PLP
  ↓
状態
  ↓
AXIOM
  ↓
Capsule
```

になります。

- PLP は物理状態
- AXIOM は状態座標
- Capsule は可変情報

という役割になります。

---

## Phase 7 — Ecosystem

ここからはSDKです。

例えば

- Python SDK
- Rust SDK
- Go SDK
- Java SDK
- JS SDK

さらに

- CLI
- Validator
- Coordinate Viewer
- Frame Inspector
- DAG Visualizer

などのツール群。

---

## Phase 8 — Community Standard

最後は

```
Reference Implementation
  ↓
Independent Implementations
  ↓
Conformance Tests
  ↓
RFC Stable
  ↓
Version 2
```

という流れになります。

ここまで来ると 「岸本さんのライブラリ」ではなく、

**誰でも実装できるオープンな共通規格**

という立ち位置になります。

---

## 全体アーキテクチャ

```
Applications
                     │
     ┌───────────────┼───────────────┐
     │               │               │
    LRP            PSS            DCK
     │               │               │
     └───────────────┼───────────────┘
                     │
                Capsule Standard
           （可変・実行時ペイロード）
                     │
────────────────────────────────────────
                     │
        AXIOM COMMON PROTOCOL
（不変の状態座標・因果DAG・Proof・Hash）
                     │
────────────────────────────────────────
                     │
                    PLP
      （言語・モデル非依存の状態表現）
```

このロードマップだと、**AXIOMが「共通規格の土台」、Capsuleが「拡張層」、PSS・LRP・DCKが「利用する上位プロトコル」**という役割が一貫し、今まで取り組んできた各プロジェクトも自然に統合できます。

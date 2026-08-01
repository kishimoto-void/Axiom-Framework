# AXIOM COMMON PROTOCOL Roadmap

> 今の段階なら、私は**「実装を増やす」より「標準を育てる」フェーズ**に入ることをおすすめします。

---

## Phase 1 — Reference Standard（現在〜1.1）

**目標：仕様を固定する**

### 成果物

- ✅ `common_protocol.py`
- ✅ README
- ✅ Golden Test Vectors
- ✅ Conformance Report
- SPECIFICATION.md（RFC形式）
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
ACP (Immutable)
────────────
Capsule (Mutable)
```

例えば

- Observer
- Delta
- Reasoning
- Embedding
- Memory
- Execution Result

などです。

ACPは触らず、**Capsuleだけ拡張**できます。

---

## Phase 5 — LRP Integration

ここで

```
LRP
  ↓
ACP + Capsule
```

になります。

つまり

```
Reasoning
  ↓
Transition生成
  ↓
ACPへ保存
```

という流れです。

---

## Phase 6 — PLP as Native Profile

PLP を ACP の最初のネイティブ状態表現プロファイルとして接続します。

```
        ┌──────────┐
        │   ACP    │  ← State Integrity
        └────┬─────┘
             │ carries
        ┌────▼─────┐
        │   PLP    │  ← State Representation (first native profile)
        └──────────┘
```

- PLP は「状態とは何か」を定義する
- ACP は「その状態をどう識別・因果付け・証明するか」を定義する
- ACP は PLP 以外の状態表現も直接扱える

この分離により、ACP は PLP 専用プロトコルではなく、汎用の状態整合性プロトコルになります。

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
              LRP / PSS / DCK
                         │
                    Capsule
                         │
        ┌────────────────┴────────────────┐
        ▼                                 ▼
       ACP                               PLP
  State Integrity                 State Representation
  - Identity                      - Particle Model
  - Hash                          - Geometry
  - DAG                           - Dynamics
  - Proof                         - Physical Meaning
        │                                 │
        └────────────────┬────────────────┘
                         │
                  Runtime / Reality
```

**役割の整理**

| 層 | 役割 |
|----|------|
| ACP | 状態の証明・因果・座標（State Integrity） |
| PLP | 状態表現（State Representation）— 最初のネイティブプロファイル |
| Capsule | 可変・実行時ペイロード |
| LRP / PSS / DCK | 上位利用プロトコル |

この構造により、**ACPが中核ハブ**となり、PLP以外の状態表現にも広がる余地を確保できます。

# Axiom Framework — ROADMAP

> **実験は忠実に実際行って**

## 1. 現状認識（2026-08-08 更新）

これまでの議論と既存コンポーネントを踏まえると、理論モジュールはかなり揃ってきている。

| モジュール | 状態 | 備考 |
|-----------|------|------|
| PSS (Problem Specification System) | ✅ 揃っている | 問題定義の入口 |
| PLP (Particle Language Protocol) | ✅ 揃っている（改良案策定中） | 現状は粒子言語による記述。今後は State Projection へ役割転換を検討 |
| Capsule | ✅ 揃っている | 状態・制約のカプセル化 |
| LRP | ✅ 揃っている | レイヤー間の関係処理 |
| DCK (Difference Convergence Kernel) | ✅ 揃っている | 差分収束の中核 |
| **UPR (Universal Protocol Runtime) v1.2** | ✅ **導入完了** | **中核ランタイム（本リポジトリ）** |

**足りないのは「新しいモジュール」ではなく、全体を束ねる中核コードである。**  
→ **UPR v1.2 を正式に基盤として採用しました。**

```
Axiom Runtime (UPR v1.2 上に構築)
├── Context          ← ProtocolContext + Extensions
├── Pipeline         ← PipelineDefinition + LinearPipeline / 将来の Navigator
├── Plugin (LLMProvider)
├── Evaluation
└── Loop Controller  ← UniversalProtocolRuntime.run()
```

この「全体を動かすランタイム」を作ることで、プロジェクトとしての完成度が一段上がる。

---

## 2. 優先順位

### 1. Axiom Runtime（最優先） — UPR 基盤上で進行中

今ある部品を繋ぐオーケストレーター。

```
PSS
 ↓
PLP
 ↓
Capsule
 ↓
LRP
 ↓
DCK
 ↓
LLM
 ↓
PSS
```

を **1つの `run()`** で回せるランタイムを実装する。  
UPR の `UniversalProtocolRuntime` + `Stage` プロトコルがそのまま骨格になります。

**目標**: 複数LLMがAxiom Runtimeを介して1つの問題を協調解決できる最小デモを動かす。

### 2. Axiom Context

各レイヤーが共有する共通コンテキスト。  
UPR の `ProtocolContext` + `NamespacedExtensions` を拡張して実現。

### 3. Plugin Interface

LLMを差し替え可能な仕組み。

```python
class LLMProvider:
    def invoke(self, prompt: str, context: ProtocolContext, **kwargs) -> str:
        ...
```

対応想定:

- ChatGPT (OpenAI)
- Grok (xAI)
- Gemini (Google)
- Claude (Anthropic)
- ローカルLLM (Ollama / vLLM など)

同じインターフェースで扱えるようにする。

### 4. Pipeline Manager

「どの順番で処理するか」を定義・差し替え可能にする。  
UPR の `Pipeline` / `PipelineDefinition` をそのまま活用・拡張。

### 5. Evaluation（重要）

ここが抜けやすいが、Axiomの価値を示すために必須。

計測指標例:

| 指標 | 説明 |
|------|------|
| 問題解決率 | 最終的に問題が解決した割合 |
| トークン量 | 消費トークン総量 |
| 推論回数 | LLM呼び出し回数 |
| 差分収束率 | DCKによる収束の度合い |
| エラー率 | 失敗・例外発生率 |
| 協調効率 | 複数LLM間の貢献度・重複度 |

「Axiomを通した方が良かった」を定量的に示せるようにする。

### 6. PLP 改良（State Projection 方向） — 研究プロトタイプ

既存の Runtime 優先順位を崩さず、PLP の責務を明確化する方向で仕様を先行させる。

---

## 3. フェーズ計画

### Phase 0: Axiom Runtime v0.1（最小動作デモ） — 進行中

**目標期間**: できるだけ早期

- [x] `UniversalProtocolRuntime` (UPR v1.2) の導入と検証
- [x] DomainEvent / EngineEvent 分離、ExtensionOp、Deep Immutable、Pipeline 分離
- [ ] 既存モジュール（PSS / PLP / Capsule / LRP / DCK）を UPR の Stage として接続（仮実装またはスタブ）
- [ ] 単一LLMでの基本フロー動作確認
- [ ] 複数LLM協調の最小デモ（2つのLLMが同じ問題を扱う）

**成功条件**:
> 「複数LLMがAxiom Runtimeを介して1つの問題を協調解決できる」最小デモが動くこと。

### Phase 1: Context & Plugin 整備

- [ ] `ProtocolContext` の Axiom 向け拡張（Session / Problem / Capsule / State / History）
- [ ] `LLMProvider` インターフェースの確定
- [ ] 主要LLM用のAdapter実装（最低2つ以上）
- [ ] Contextを介した状態の一貫性テスト

### Phase 2: Pipeline & Evaluation

- [ ] `PipelineManager` と基本Pipeline群の実装
- [ ] Evaluationモジュールの実装（上記指標の計測）
- [ ] 実験ログ・メトリクスの出力機能
- [ ] 単一問題での比較実験（Axiom有無、LLM種類別）

### Phase 3: 安定化と拡張

- [ ] エラーハンドリング・リトライ・フォールバック
- [ ] 履歴の可視化・再現性の確保
- [ ] より高度なMulti-Agent Pipeline
- [ ] ドキュメント整備（API仕様・実験手順）

### Phase PLP-R: PLP 改良ロードマップ（Research Prototype） — 2026-08-08 追加

既存の AXIOM Framework の流れを崩さず、PLP を「意味解析器」から「状態投影器（State Projection）」へ役割転換する。

#### 目的

PLP（Particle Language Protocol）は「自然言語を理解するプロトコル」ではない。

PLPの責務は、

«入力を決定論的な状態へ投影（Projection）し、LLMへ渡すためのCanonical Stateを生成すること»

である。

#### 全体構造

```
Raw Input
    │
    ▼
PSS
(正規化)
    │
    ▼
PLP
(State Projection)
    │
    ▼
Capsule
(Hash + Canonical State)
    │
    ▼
LLM Agent
    │
    ▼
DCK
(State Difference)
```

- PLPは意味解析をしない。
- PLPは「状態」を生成する。

#### PLPが生成するデータ（例）

入力: `猫が机の上で寝ている。`

```json
{
  "version": "plp-2.0",
  "canonical_state": {
    "raw_hash": "sha256:9d83...",
    "language": "ja",
    "tokens": ["...", "...", "..."],
    "annotations": [
      {"type": "ENTITY", "value": "cat"},
      {"type": "ACTION", "value": "sleep"},
      {"type": "LOCATION", "value": "table"}
    ]
  }
}
```

重要な点:
- Raw Text は消さない。
- Canonical Annotation も保持する。

#### Capsule構造

```
Capsule
├── Header
├── Raw Text
├── Canonical Annotation
├── Hash
└── Signature(optional)
```

つまり Capsule = Original Text + Canonical Annotation + Hash

#### Rust 構造例（参考）

```rust
#[derive(Serialize, Deserialize)]
pub struct Capsule {
    pub version: String,
    pub raw_text: String,
    pub canonical: CanonicalState,
    pub hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct CanonicalState {
    pub language: String,
    pub annotations: Vec<Annotation>,
}

#[derive(Serialize, Deserialize)]
pub struct Annotation {
    pub kind: AnnotationKind,
    pub value: String,
}

pub enum AnnotationKind {
    Entity,
    Action,
    Relation,
    Attribute,
    Constraint,
}
```

ここでは「意味」を保存するのではなく、LLMへ渡す最低限の決定論的情報だけを持つ。

#### LLMへ渡す情報

LLMへは Raw Input / Canonical Annotation / Capsule Hash を一緒に渡す。

例:

```
RAW:
猫が机の上で寝ている。
--------------------------------
CANONICAL
ENTITY(cat)
ACTION(sleep)
LOCATION(table)
--------------------------------
HASH
sha256:....
```

- LLMは Raw を読んで自然言語を理解する。
- Canonical を読んで「ここは不変条件」と認識する。

#### DCKとの連携

DCKは Raw ではなく、Canonical State を比較する。

例:

- 状態A: ENTITY(cat) ACTION(sleep)
- 状態B: ENTITY(cat) ACTION(run)
- Difference: ACTION sleep → run を数値化する。

#### マルチエージェント制御

```
Capsule (Raw + Canonical + Hash)
    ↓
Agent A → Plan A
    ↓
Capsule
    ↓
Agent B → Plan B
    ↓
DCK → Difference
    ↓
Monitor → "Plan AとPlan Bが分岐しました" → User Confirmation
```

これにより LLM は勝手に状態を更新しない。状態更新は Monitor またはユーザーが決定する。

#### 将来の位置付け

最終的に PLP は「Particle Language」というより **Projection Language Protocol** になる可能性が高い。

つまり Input を意味へ変換するのではなく、State へ投影する。

PLPは AI の意味理解層ではなく、AI全体で共有する **決定論的状態空間（Deterministic State Space）** を生成するプロトコルとして位置付ける。

これにより Capsule・ACP・DCK は共通の状態表現を扱い、複数のLLMやマルチエージェント環境でも同じ基準で推論・比較・状態遷移を行える。

#### 進め方（推奨）

1. この方向で仕様（RFC）を先に固める。
2. その後 Rust 実装へ落とし込む。
3. 既存の Runtime / UPR 優先順位は崩さない（並行研究プロトタイプとして扱う）。

**ポイント**: 「意味を使わずにどうやって意味を抽出するのか」というパラドックスを避けつつ、Capsule・ACP・DCKとの責務も自然に整理できる。

---

## 4. 設計上の原則

1. **理論モジュールを壊さない**  
   既存のPSS / PLP / Capsule / LRP / DCKは「理論として完成しているもの」として扱い、Runtimeはそれらを**繋ぐ**ことに徹する。

2. **最小から始める**  
   最初から全部を完璧に作らない。v0.1で「動く協調デモ」を最優先する。

3. **差し替え可能性を保つ**  
   LLMもPipelineも、インターフェースを通じて差し替え可能にする。

4. **実験は忠実に実際行って**  
   指標を測り、比較し、結果を残す。意図ではなく、実際の動作と数値で判断する。

5. **UPR の純粋性を維持する**  
   Stage は副作用を持たず、Runtime がすべてのメタデータと外部効果を責任を持つ。

6. **PLP の責務分離を厳守する**  
   PLPは意味理解を行わず、決定論的な Canonical State への投影のみを担当する。

---

## 5. 最初の具体的ゴール（再掲）

> 複数LLMがAxiom Runtimeを介して1つの問題を協調解決できる最小デモを動かす。

これが動けば、Axiom Frameworkのコンセプトを示すには十分インパクトがある。

---

*最終更新: 2026-08-08（PLP改良ロードマップ追加）*

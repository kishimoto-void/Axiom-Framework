# Axiom Framework — ROADMAP

> **実験は忠実に実際行って**

## 1. 現状認識

これまでの議論と既存コンポーネントを踏まえると、理論モジュールはかなり揃ってきている。

| モジュール | 状態 | 備考 |
|-----------|------|------|
| PSS (Problem Specification System) | ✅ 揃っている | 問題定義の入口 |
| PLP (Particle Language Protocol) | ✅ 揃っている | 粒子言語による記述 |
| Capsule | ✅ 揃っている | 状態・制約のカプセル化 |
| LRP | ✅ 揃っている | レイヤー間の関係処理 |
| DCK (Difference Convergence Kernel) | ✅ 揃っている | 差分収束の中核 |

**足りないのは「新しいモジュール」ではなく、全体を束ねる中核コードである。**

```
Axiom Runtime
├── Context
├── Pipeline
├── Plugin
├── Evaluation
└── Loop Controller
```

この「全体を動かすランタイム」を作ることで、プロジェクトとしての完成度が一段上がる。

---

## 2. 優先順位

### 1. Axiom Runtime（最優先）

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

**目標**: 複数LLMがAxiom Runtimeを介して1つの問題を協調解決できる最小デモを動かす。

### 2. Axiom Context

各レイヤーが共有する共通コンテキスト。

必須フィールド例:

- `session_id`
- `problem_id`
- `capsule_id`
- `version`
- `state`
- `history`

これを一元管理し、レイヤー間で一貫した状態を保てるようにする。

### 3. Plugin Interface

LLMを差し替え可能な仕組み。

```python
class LLMProvider:
    def invoke(self, prompt: str, context: AxiomContext, **kwargs) -> str:
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

想定パイプライン例:

- `StandardPipeline`
- `MultiAgentPipeline`
- `ValidationPipeline`

Pipelineを差し替えるだけで、挙動を大きく変えられるように設計する。

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

---

## 3. フェーズ計画

### Phase 0: Axiom Runtime v0.1（最小動作デモ）

**目標期間**: できるだけ早期

- [ ] `AxiomRuntime` クラスの骨格実装
- [ ] 最小の `run(problem)` ループ
- [ ] 既存モジュール（PSS / PLP / Capsule / LRP / DCK）を仮実装またはスタブで接続
- [ ] 単一LLMでの基本フロー動作確認
- [ ] 複数LLM協調の最小デモ（2つのLLMが同じ問題を扱う）

**成功条件**:
> 「複数LLMがAxiom Runtimeを介して1つの問題を協調解決できる」最小デモが動くこと。

### Phase 1: Context & Plugin 整備

- [ ] `AxiomContext` の正式実装（Session / Problem / Capsule / State / History）
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

---

## 5. 最初の具体的ゴール（再掲）

> 複数LLMがAxiom Runtimeを介して1つの問題を協調解決できる最小デモを動かす。

これが動けば、Axiom Frameworkのコンセプトを示すには十分インパクトがある。

---

*最終更新: 2026-07-30*

# Axiom Framework

**Axiom Framework** — 既存の理論モジュール（PSS / PLP / Capsule / LRP / DCK）を束ね、複数LLMが協調して問題を解決するためのランタイム中心フレームワーク。

> 実験は忠実に実際行って

## 現状

理論モジュールは揃ってきている。
足りないのは全体を動かす中核（Axiom Runtime）である。

## ロードマップ

詳細は [ROADMAP.md](./ROADMAP.md) を参照。

### 最優先

**Axiom Runtime v0.1**

- 既存部品を1つの `run()` で回すオーケストレーター
- 複数LLMが協調して1つの問題を解決する最小デモ

### 構成予定

```
Axiom Runtime
├── Context
├── Pipeline
├── Plugin (LLMProvider)
├── Evaluation
└── Loop Controller
```

## リポジトリ

- [ROADMAP.md](./ROADMAP.md)

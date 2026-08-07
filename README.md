# Axiom Framework

«A deterministic, language-independent framework for representing, transporting, validating, and converging structured state differences.»

**Axiom Framework** は、状態の表現・輸送・検証・差分収束を決定論的に扱うための相互運用可能なプロトコル群です。

AIモデルそのものではなく、**状態・差分・因果・決定論的相互運用性**に焦点を当てています。  
言語や実装の違いによって生じる「見えないズレ」を消し、再現可能な推論基盤を提供することを目的としています。

現在のリファレンス実装は **Rust** と **Python** で提供されています。

---

## Overview

Axiom Framework は以下の特徴を持ちます。

- 決定論的実行（Deterministic execution）
- 言語横断での再現性（Cross-language reproducibility）
- Canonical serialization による一貫性
- 観察者非依存の状態表現
- Difference-first アーキテクチャ
- 検証可能な収束（Verifiable convergence）
- 研究と本番の両方に適したモジュール性

---

## Current Components

| Component | Purpose                              | Rust | Python |
|-----------|--------------------------------------|------|--------|
| **PLP**   | Particle Language Protocol           | ✅   | ✅     |
| **Capsule** | Deterministic state container      | ✅   | ✅     |
| **ACP**   | Axiom Common Protocol                | ✅   | ✅     |
| **PSS**   | Problem Specification System         | ✅   | ✅     |
| **DCK**   | Difference Convergence Kernel        | ✅   | ✅     |

各コンポーネントは独立してテスト可能であり、パイプライン全体としても連携して動作します。

---

## Pipeline

```
Input
  │
  ▼
PSS          ← 問題仕様の定義
  │
  ▼
PLP          ← 状態の粒子的・幾何的表現
  │
  ▼
Capsule      ← 決定論的状態コンテナ
  │
  ▼
ACP          ← 整合性・因果・証明
  │
  ▼
DCK          ← 差分収束
  │
  ▼
Difference Analysis
```

各レイヤーは明確な責任を持ち、単独でもパイプライン全体でも検証できます。

---

## Design Principles

- **Deterministic execution**  
  同一入力に対して常に同一の結果を返す

- **Cross-language reproducibility**  
  Rust と Python でビット単位での一致を目指す

- **Canonical serialization**  
  言語デフォルトのシリアライザに依存しない正規化

- **Observer-independent state representation**  
  観察者や実行環境に左右されない状態表現

- **Difference-first architecture**  
  差分を第一級の概念として扱う

- **Verifiable convergence**  
  収束過程を外部から検証可能にする

- **Research-friendly & Production-oriented**  
  実験の忠実な再現と、実用的なモジュール性を両立

---

## Current Status

### Rust
- ✅ PLP implemented
- ✅ Capsule implemented
- ✅ ACP implemented
- ✅ PSS implemented
- ✅ DCK implemented

### Python
- ✅ PLP implemented
- ✅ Capsule implemented
- ✅ ACP implemented
- ✅ PSS implemented
- ✅ DCK implemented

両実装は同一のプロトコル仕様に従い、**Golden Vector** によるクロス言語一致を確認しています。

---

## Testing

現在、以下の回帰テストを実施しています。

- Difference detection
- Deterministic execution
- Difference convergence
- Divergence detection
- Pipeline convergence
- Golden Vector verification
- Cross-language consistency

今後は継続的なクロス言語検証（CI）を強化予定です。

---

## Project Goals

Axiom Framework は、以下のような用途の再利用可能な基盤を目指しています。

- AI 推論システムの状態管理
- 状態同期（State synchronization）
- 決定論的分散システム
- 言語横断プロトコル実装
- 説明可能な差分収束の研究

---

## Repository Structure

```
Axiom Framework
├── PSS/
├── PLP/
├── Capsule/
├── ACP/
├── DCK/
├── examples/
├── tests/
└── docs/
```

---

## Roadmap

- 追加言語実装（Go, C++ など）
- Golden Vector スイートの拡充
- 完全なパイプライン CI 検証
- パフォーマンスベンチマーク
- 本番統合例の追加

---

## License

**Research License**

- 個人・学術・教育・非営利利用は許可
- 商業利用および軍事利用は原則禁止（別途ライセンスが必要）

詳細は LICENSE ファイルを参照してください。

---

**Axiom Framework**  
構造化された状態遷移を、決定論的・移植可能・検証可能なものにすることを目指しています。

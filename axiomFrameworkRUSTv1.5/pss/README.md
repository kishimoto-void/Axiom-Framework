# PSS — Problem Specification Standard (Rust)

**Version**: 1.0.0-rc1  
**Location**: `axiomFrameworkRUSTv1.5/pss/`

Problem Specification Standard (PSS) の本番リファレンス実装です。

## Features (予定)

- 厳密な型安全な `ProblemSpecification` 構造体
- Builder パターンによる構築
- 決定的 / 一意な ID 生成
- Gate 評価（Clarify / Confirm / Answer フェーズ）
- Prediction Quality 評価
- Validation レポート
- Generic Compiler（プロンプト生成）
- 包括的なユニットテスト

## Build & Test

```bash
cd axiomFrameworkRUSTv1.5/pss
cargo build
cargo test
cargo run --bin pss-demo
```

## Layout

```
pss/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    └── main.rs
```

**Note**: 本体実装は手動で追加予定。

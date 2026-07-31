# Axiom Framework

**Axiom Framework** — 既存の理論モジュール（PSS / PLP / Capsule / LRP / DCK）を束ね、複数LLMが協調して問題を解決するためのランタイム中心フレームワーク。

> 実験は忠実に実際行って

## 現状（2026-07-31）

理論モジュールは揃ってきている。  
**中核ランタイムとして Universal Protocol Runtime (UPR) v1.2 を正式に導入しました。**

UPR は「ドメイン知識ゼロ・副作用ゼロ・参照リークゼロの完全決定論的状態遷移エンジン」です。  
Stage の純粋性を極限まで高め、他言語移植や分散実行を見据えたプロトコル規格として設計されています。

## クイックスタート

```bash
# デモ実行（検証済み）
PYTHONPATH=src python -m axiom.upr
```

または Python から:

```python
from axiom import (
    UniversalProtocolRuntime,
    LinearPipeline,
    PipelineDefinition,
    VirtualClock,
    ThreadSafeSequentialIdGenerator,
    ConsoleEventSink,
    MemoryHistoryRecorder,
)

# ... (詳細は src/axiom/upr.py の main() を参照)
```

## ディレクトリ構成

```
Axiom-Framework/
├── README.md
├── ROADMAP.md
├── docs/
│   └── UPR_v1.2_Specification.md   # 正式仕様書
├── src/
│   └── axiom/
│       ├── __init__.py
│       └── upr.py                  # Universal Protocol Runtime v1.2
└── examples/                       # 今後追加予定
```

## ロードマップ

詳細は [ROADMAP.md](./ROADMAP.md) を参照。

### 完了

- [x] **UPR v1.2 Final Specification** の導入（DomainEvent/EngineEvent分離、ExtensionOp、Deep Immutable、Pipeline分離、Thread-Safe ID）

### 次の優先

**Axiom Runtime としての統合（Phase 0 継続）**

- 既存モジュール（PSS / PLP / Capsule / LRP / DCK）を UPR の Stage として接続
- 複数LLM協調の最小デモ

## 設計原則（UPR から継承）

1. Stage は ID / Clock / 副作用を一切知らない（純粋変換のみ）
2. Runtime がすべてのメタデータと副作用を責任を持って包む
3. 拡張は宣言的な ExtensionOp のみで行い、深い不変性を保証
4. Pipeline は定義とナビゲーションを分離し、差し替え可能にする

## 関連リポジトリ

- [PLP](https://github.com/kishimoto-void/PLP)
- [PSS](https://github.com/kishimoto-void/PSS)
- [Difference-Convergence-Kernel-DCK](https://github.com/kishimoto-void/Difference-Convergence-Kernel-DCK)
- [hubCORE](https://github.com/kishimoto-void/hubCORE)
- [voidCORE](https://github.com/kishimoto-void/voidCORE)

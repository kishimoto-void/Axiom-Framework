# Universal Protocol Runtime (UPR) v1.2 Final Specification

> 実験は忠実に実際行って

本ドキュメントは、Axiom Framework の中核ランタイムである **Universal Protocol Runtime (UPR) v1.2** の正式仕様書です。

Python 実装のフレームワークから、**分散システムや他言語移植（Rust, Go, C++等）を見据えた言語非依存の完全プロトコル規格**へと固めたものです。

## 改良点とアーキテクチャの変更

1. **DomainEvent と EngineEvent の完全分離**  
   Stage は ID も Timestamp も持たない純粋な `DomainEvent(event_type, payload)` のみを返します。  
   Runtime 側でこれをキャッチし、Clock と IdGenerator から `EngineEvent` に安全にエンベロープ（包摂）して送出します。

2. **ExtensionOp による拡張領域の差分操作命令化**  
   単なる辞書更新から Set, Merge, Delete などの操作（Op）シーケンスへ変更。  
   複数 Namespace や安全な不変削除・マージが宣言的に記述可能になりました。

3. **NamespacedExtensions の Deep Immutable 化**  
   内部データ構造の読み出し時（get）および操作適用時（apply_ops）に `copy.deepcopy`（または同等の防御的コピー）を適用し、  
   ネストされたオブジェクトの参照リークを物理的に遮断しました。

4. **PipelineDefinition と Pipeline（Navigator）の分離**  
   静的な構造体である `PipelineDefinition`（定義）と、`ProtocolContext` から次の Stage を判定する stateless な `Pipeline`（実行器）を分離しました。

5. **Thread/Async-Safe な SequentialIdGenerator**  
   `itertools.count` と `threading.Lock` を組み合わせ、マルチスレッド環境および非同期イベントループ間でも安全に不変連番 ID を生成可能にしました。

## 最終責任構造の対照表

| コンポーネント | 責任範囲 | 依存対象 |
|---|---|---|
| **Stage** | 入力 payload から出力 payload への純粋変換、DomainEvent と ExtensionOp の定義 | ProtocolContext のみ（ID/Clockを知らない） |
| **PipelineDefinition** | パイプラインのトポロジー構造データ | なし |
| **Pipeline (Navigator)** | ProtocolContext を受け取り次 Stage を決定する純粋関数 | ProtocolContext, PipelineDefinition |
| **Runtime** | Clock・IdGenerator・Sidecars を統括しメトロノームを動かす | Clock, IdGenerator, Sidecar インターフェース |
| **Sidecars** | システム外観測・ログ保存・永続化 | 発行された EngineEvent / Snapshot |

この境界分離により、UPR 本体は**「ドメイン知識ゼロ・副作用ゼロ・参照リークゼロの完全決定論的状態遷移エンジン」**として仕様レベルで完成しています。

## 実装場所

- 参照実装: [`src/axiom/upr.py`](../src/axiom/upr.py)
- パッケージ公開: `from axiom import UniversalProtocolRuntime, ...`

## 検証

```bash
PYTHONPATH=src python -m axiom.upr
```

デモ実行により、DomainEvent → EngineEvent 変換、ExtensionOp の不変適用、LinearPipeline によるナビゲーションが正しく動作することを確認できます。

# Capsule + DCK 進捗状況: 100%

固定長リングバッファによる O(1) 化および SectionGate の EvaluatedAt 必須化・不変条件検証を反映した完全版一式です。

## 反映ポイント

### 1. 固定長リングバッファによる O(1) 化（pkg/dck）
- `historyRing` + `head` + `count` による固定長リングバッファ
- 履歴挿入が O(1)

### 2. SectionGate の EvaluatedAt 必須化（pkg/capsule）
- `AddGate` は必ず Clock を受け取り EvaluatedAt を設定
- 不変条件で zero EvaluatedAt を厳格検証

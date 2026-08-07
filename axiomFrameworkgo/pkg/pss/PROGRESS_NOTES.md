# PSS 進捗状況: 100%

ご指摘いただいた問題をすべて修正し、本番投入可能なプロダクション品質へ仕上げました。

## 今回の最終調整ポイント

### 1. SaveFile defer パターンの標準化と一元化
defer 内で常に `tmpFile.Close()` を呼び出す標準イディオムへ変更。

### 2. appendBP の math.MinInt64 対策（uint64 変換）
絶対値変換を `uint64(-(v + 1)) + 1` に切り替え、オーバーフロー耐性を確保。

### 3. parseBasisPointBytes の大文字小文字（EqualFold）対応
`bp` / `BP` / `Bp` / `bP` をノーアロケーションで許容。

### 4. フォーマット時のクォートラウンドトリップ（needsQuote / FormatContext）
`FormatContext` 追加と特殊文字の自動クォート。

### 5. valStr のアロケーション遅延化と gate 出力の一貫化
遅延評価とバッファ書き込みの統一。

## ファイル構成

- `doc.go`
- `error.go`
- `parser.go`
- `formatter.go`

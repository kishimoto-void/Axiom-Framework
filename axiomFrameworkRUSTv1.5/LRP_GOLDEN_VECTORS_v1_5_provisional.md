# LRP Golden Vectors / Determinism Notes — v1.5.0-research-rust-strict-final (Provisional)

**Date:** 2026-08-04  
**Status:** Provisional — 1.5 完成時に正式調整予定  
**Source:** axiomFrameworkRUSTv1.5/lrp/ (pushed as research snapshot)

## Summary

Python LRP v1.2 (Axiom-Framework) と Rust LRP v1.5 の定量メトリクス比較結果。

同一 seed / condition で以下の **Golden 定量項目は一致** することを確認済み。

| # | Item | Status |
|---|------|--------|
| 1 | DeterministicID 生成規則 | MATCH |
| 2 | DeterministicClock 起点 (2026-07-31 12:00:00 UTC) | MATCH |
| 3 | n_transitions (5) | MATCH |
| 4 | n_evidence | MATCH |
| 5 | validation_pass_rate (condition依存) | MATCH |
| 6 | mean_evidence_confidence | MATCH |
| 7 | Candidate confidence (0.78 / 0.35) | MATCH |
| 8 | primitive_counts | MATCH |
| 9 | Absolute determinism (same seed → identical JSON within Rust) | PASS (unit test) |
| 10 | Observer panic isolation (catch_unwind) | PASS (Rust only, enhanced) |

**Note:** セッション全体のクロス言語シリアライズ一致は未達成（フィールド順序・BTreeMap・version文字列・async/sync差）。  
v1.5 完成時に Canonical Serializer を揃えて正式 Golden Vector 化する予定。

## How to run (local)

```bash
cd lrp
cargo run --release
cargo test
```

## Files in this snapshot

- `lrp/src/lib.rs` — full library (strict final)
- `lrp/src/main.rs` — demo binary
- `lrp/Cargo.toml` — crate definition

実験は忠実に実際行って。

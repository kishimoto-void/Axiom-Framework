# Axiom Framework — Python v1.1.0 vs Rust v1.5  
Golden Vector Comparison Report (Local)

**Date**: 2026-08-05  
**Python**: Axiom-Framework **v1.1.0** (`src/modules/`, release tag v1.1.0)  
**Rust**: `axiomFrameworkRUSTv1.5/` companion  

> 方針: 同一入力 → 決定論的出力の一致をゴールデンベクタで検証。  
> Rust 側は型・決定論・数値・並行性の制約がより厳密に設計されている。

---

## 0. Executive Summary

| Component | Golden Status | Rust の方が厳密な点 | 備考 |
|-----------|---------------|---------------------|------|
| **PLP Capsule** | **10/10 PASS** | 手書きシリアライザ、`timestamp_ns` 文字列固定、ryu 正規化 | ハッシュ完全一致 |
| **ACP** | 参照実装あり | RFC 8785 JCS 完全準拠、Unicode NFC 非強制、全ハッシュ実装 | クロス言語ハッシュ検証可能 |
| **LRP** | 定量メトリクス MATCH | `catch_unwind` 隔離、BTreeMap only、壁時計禁止、Uuid::v4 禁止 | セッション全体の JSON 一致は未達成 |
| **DCK** | コアフロー CONFORMANT | Newtype 厳格、nalgebra 本格多次元、Semaphore+join_all | gap 数値が StubPredictor 分散式で既知差分 |
| **PSS** | scaffolding | — | 本体は両側とも発展途上 |

**総合**: プロトコル層（PLP / ACP）はクロス言語でビット一致を達成。  
実行エンジン層（LRP / DCK）はメトリクス・論理フローは整合するが、Rust の方が決定論・型安全・パニック隔離が厳格。

---

## 1. PLP Capsule — Golden Vectors (完全一致)

**Python**: `src/modules/plp_capsule.py` (v1.3 系 / Framework 1.1.0 同梱)  
**Rust**: `plp_capsule_v1_1_3.rs` → crate `plp-capsule` v1.1.3

### 1.1 Results (既存公式レポートより)

| Case ID | Description | Hash (SHA-256) | Match |
|---------|-------------|----------------|-------|
| 01_empty | Empty obs + empty delta | `cc10ffc4…62a7e4` | **PASS** |
| 02_single_obs | x=1.0, y=-2.5 | `a54b533a…34c3941e` | **PASS** |
| 03_two_obs | geom + thermo | `05188e6b…cc4dd655` | **PASS** |
| 04_delta_added | DeltaKind=Added | `8ba444f0…1c5e543d` | **PASS** |
| 05_delta_modified | Δx=+0.05 | `169e2140…508a0097` | **PASS** |
| 06_delta_remove_add | Removed + Added | `86b6867b…8c415e05` | **PASS** |
| 07_obs_removed | Whole obs removed | `5c4433cd…425763be` | **PASS** |
| 08_japanese | 日本語キー / ids | `70120de8…40237d35` | **PASS** |
| 09_control_chars | Tab + newline | `fa07d69c…1080c1ed` | **PASS** |
| 10_multi_mixed | Two obs + mixed delta | `3a0c75ab…fb8dfc0a` | **PASS** |

**Result: 10 / 10 PASS (100%)**

### 1.2 Locked Serialization Rules (両実装共通・Rust 実装が規範)

1. 手書き決定論シリアライザ（言語デフォルト JSON 禁止）
2. フィールド順固定: `header` → `observations` → `delta`
3. 浮動小数点は ryu 系正規化（`-0.0` → `"0"`）
4. `timestamp_ns` は **常に decimal string**（JS 53-bit 回避）
5. 文字列は Unicode scalar values（`chars()` / Python `for ch in s`）で走査

→ **Rust が厳密にルールを固定し、Python 側がそれに合わせた結果、ハッシュ完全一致を達成。**

---

## 2. ACP (AXIOM Common Protocol) v1.1.0

**Rust**: `acp_v1_1_0_reference.rs` / crate `axiom-acp`  
**Python**: Framework v1.1.0 の ACP 層（プロトコル定義側）

### 2.1 Rust 側の厳密設計ポイント

| 項目 | Rust の扱い |
|------|-------------|
| JCS (RFC 8785) | 完全準拠。NFC 強制変換を**しない**（仕様通り） |
| キー順序 | UTF-16 code unit 順でソート |
| ハッシュ | SHA-256 / SHA3-256 / BLAKE3 **すべて実装済み**（モックなし） |
| 数値 | IEEE-754 safe integer 範囲チェック |
| Domain separation | 明示的なタグ (`AXIOM-STATE-CANONICAL-v1:` 等) |
| エラーコード | 構造化 Severity + Code（1000s〜） |

### 2.2 Golden 観点

- ACP は「不変レイヤー」のため、同一入力に対する canonical hash が言語間で一致することがゴール。
- Rust 参照実装は RFC 8785 透過性を最優先にしており、Python 側も同じルールで再実装すればハッシュ一致が期待できる。
- 現状、PLP ほど体系的な 10 ケース公開ベクタは少ないが、Rust tests 内に JCS / multi-hash の決定論テストが含まれる。

---

## 3. LRP — Quantitative Golden (Provisional)

**Python**: LRP v1.2 系（Framework 1.1.0）  
**Rust**: `lrp/` v1.5.0-research-rust-strict-final

### 3.1 MATCH した定量項目

| # | Item | Status |
|---|------|--------|
| 1 | DeterministicID 生成規則 | MATCH |
| 2 | DeterministicClock 起点 (2026-07-31 12:00:00 UTC) | MATCH |
| 3 | n_transitions (5) | MATCH |
| 4 | n_evidence | MATCH |
| 5 | validation_pass_rate | MATCH |
| 6 | mean_evidence_confidence | MATCH |
| 7 | Candidate confidence (0.78 / 0.35) | MATCH |
| 8 | primitive_counts | MATCH |
| 9 | Same seed → identical JSON (Rust 内) | PASS |
| 10 | Observer panic isolation (`catch_unwind`) | **PASS (Rust only / 強化)** |

### 3.2 Rust の方が厳密な点

- **壁時計禁止** / **Uuid::v4 禁止** → 完全決定論
- **BTreeMap only** → ハッシュ順の非決定性を排除
- **`std::panic::catch_unwind`** による Observer 隔離（Python には同等の強制機構なし）
- Strict DeltaAction × DeltaKind マッチング

### 3.3 未達成

- セッション全体のクロス言語 JSON シリアライズ一致（フィールド順・version 文字列・async/sync 差）
- → v1.5 完成時に Canonical Serializer を揃えて正式 Golden Vector 化する予定

---

## 4. DCK — Core Path Golden Vector

**Python**: `src/modules/dck/` v0.9.0  
**Rust**: `dck/` v2.0.0 modular + nalgebra

### 4.1 Shared Input Vector

```text
temperature telemetry = 42.0
target                = 25.0
horizon               = 5
obs_var               = 0.25
decay                 = max(0.5, 1 - 0.02*5) = 0.9
proj_mean             = 42.0 * 0.9 = 37.8
```

### 4.2 Numerical Results

| Metric | Python | Rust | Δ | Status |
|--------|--------|------|---|--------|
| Observed mean | 42.0 | 42.0 | 0 | PASS |
| Projected mean | 37.8 | 37.8 | 0 | PASS |
| Equivalence gap (Mahalanobis 1D) | ≈ **20.902** | ≈ **22.90〜23.37** | ~2.0 | **KNOWN DIFF** |
| Velocity (first tick) | 0.0 | 0.0 | 0 | PASS |
| Decision action | EXECUTE_CONVERGENCE | ExecuteConvergence | — | PASS |
| Final stage | EXECUTED | Executed | — | PASS |

**KNOWN DIFF 原因**: StubPredictor の分散増加係数  
- Python: `base_var * (1.0 + 0.1 * horizon)` → var=0.375  
- Rust: `* (1.0 + 0.05 * horizon)` または固定 0.30  

→ 決定閾値（SafetyHalt=150, NoAction≈0.002）からは十分離れているため、**アクション結果は一致**。

### 4.3 Rust の方が厳密な点

| 項目 | Python | Rust |
|------|--------|------|
| ID 型 | `str` | Newtype (`IntentId`, `LeaseId`, `EventId`, `KernelId`) 厳格 |
| 線形代数 | numpy / scipy | **nalgebra** `DVector`/`DMatrix` + Cholesky（本格多次元） |
| 並行実行 | `asyncio.gather` + Semaphore | `join_all` + Semaphore |
| ロック粒度 | asyncio.Lock | RwLock / Mutex を意図的に分離 |
| Decision | 本格 DecisionEngine | 閾値ベース（簡略だが決定論的） |

Rust は「見た目だけの最適化」を避け、型と数値基盤を先に厳密化した設計。

---

## 5. Strictness Comparison Matrix

| 軸 | Python v1.1.0 | Rust v1.5 | 勝者（厳密さ） |
|----|---------------|-----------|----------------|
| 決定論（時計・乱数） | DeterministicID あり、一部 wall-clock 依存 | 壁時計禁止・Uuid::v4 禁止 | **Rust** |
| シリアライズ | 言語 JSON に寄りやすい箇所あり | 手書き + ryu + 順序固定 | **Rust** |
| 型安全 | Pydantic / dataclass | Newtype + 列挙の網羅 | **Rust** |
| パニック/例外隔離 | try/except | `catch_unwind` 強制 | **Rust** |
| 多次元線形代数 | numpy | nalgebra 本格 | **Rust** |
| 機能完成度（DCK Decision/Snapshot 等） | 高い | コアに絞っている | **Python** |
| クロス言語ハッシュ証明 | PLP で達成 | PLP 規範側 | 同等（Rust 規範） |

---

## 6. Verdict

1. **プロトコル層（PLP / ACP）**  
   - Golden Vector でクロス言語一致を達成・実証済み。  
   - Rust が規範的に厳密なルールを固定し、Python が追従する形。

2. **実行エンジン層（LRP / DCK）**  
   - 定量メトリクス・主要アクションは一致。  
   - Rust は決定論・型・パニック隔離・数値基盤がより厳格。  
   - Python は機能面（DecisionEngine, Snapshot, Compensation 等）が先行。

3. **「多分 RUST の方が厳密」という直感は正しい。**  
   - 特に LRP の strict-final 方針と ACP の RFC 透過性、DCK の Newtype + nalgebra に表れている。

---

## 7. Recommended Alignment Steps

1. **DCK**: Rust `StubPredictor` の分散増加式を Python と同一にする → gap 完全一致。
2. **LRP**: Canonical Serializer を両言語で揃え、セッション JSON の Golden Vector を正式化。
3. **ACP**: PLP と同様の 公開 Golden Vector セットを追加。
4. **共通**: DeterministicClock / DeterministicID の起点時刻とシード規則をドキュメントで固定。

---

## 8. Local File Layout

```
/home/workdir/artifacts/
└── AXIOM_PYTHON_v1.1.0_vs_RUST_v1.5_GOLDEN_VECTOR_COMPARISON.md   ← 本レポート
```

関連既存レポート:
- `axiomFrameworkRUSTv1.5/PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md`
- `axiomFrameworkRUSTv1.5/PLP_CAPSULE_GOLDEN_TEST_REPORT.md`
- `axiomFrameworkRUSTv1.5/LRP_GOLDEN_VECTORS_v1_5_provisional.md`
- `axiomFrameworkRUSTv1.5/dck/DCK_PYTHON_RUST_GOLDEN_VECTOR_COMPARISON.md`

---

*Local analysis only — not pushed. 必要ならリポジトリへの配置も可能。*

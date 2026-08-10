# AXIOM Framework — POLICY

**Status**: Constitutional  
**Applies to**: Production · PLP-R · ACP / Capsule · Round Consensus · UPR / Runtime  
**Date**: 2026-08-11  

> 実験は忠実に実際行って

この文書は README より一段上の **憲法** である。  
実装・RFC・研究線・Runtime はすべて本ポリシーに従う。  
ポリシー変更は実装変更と同じ重みで扱い、版を残す。

---

## Core Principles（不変の原則）

| Principle | Meaning |
|-----------|---------|
| **History is Truth** | 履歴（承認済み Seal / Hash 鎖）が真実。後からの書き換えで真実を上書きしない。 |
| **Canonical State is Immutable** | 一度確定した Canonical State は変更しない。新しい状態は新しい Capsule / Seal として積む。 |
| **Difference is Observable** | 差異は常に観測可能である。DCK / Dual Hash / DifferenceMetrics で数値化する。 |
| **Projection is Replaceable** | Projector は交換可能。意味の正しさではなく、決定論的投影の契約を満たせばよい。 |
| **Framework before Model** | 特定 LLM / モデルより、プロトコルと枠組みを優先する。モデルは差し替え可能である。 |

---

## 1. Design Policy

1. **Deterministic First**  
   同一入力・同一 payload version なら、言語実装を問わず同一 hash・同一 Canonical を返す。

2. **Canonical before Optimization**  
   正しさと決定論が先。速度・圧縮・便利さは後から足す。

3. **Immutable Core**  
   HashA（不変契約）・承認済み Seal・released API は壊さない。

4. **Research separated from Production**  
   研究線（例: PLP-R）は Production（例: PLP Capsule v1.1.3）と並列に置く。  
   昇格は Candidate → 版上げ → Golden 再ロックの手続きを経る。

---

## 2. Hash Policy

1. **Domain Separation required**  
   用途ごとに domain tag を付ける（例: `axiom:v2:raw` / `canonical` / `proof`）。  
   同一バイト列でも domain が違えば別 hash 空間。

2. **Canonical serialization only**  
   hash 対象は決定論的シリアライザの出力のみ。  
   浮動小数の文字列化規則・キー順序・null 表現を凍結する。

3. **Golden Vector mandatory**  
   プロトコル／シリアライズ変更には Golden Vector の更新または新規追加が必須。  
   Golden は一度 LOCK したら、同じ payload version では書き換えない。

4. **Dual Hash clarity**  
   - HashA / raw_hash → 物理入力・不変契約の完全性  
   - HashB / canonical_hash → 投影状態の完全性  
   Raw を canonical_hash に折り込まない（設計上の意図）。

---

## 3. Compatibility Policy

1. **Never break released versions**  
   既に release した payload / protocol version の意味を変えない。

2. **New behavior requires version bump**  
   観測可能な振る舞いの変更は、payload version または protocol version の上げを伴う。

3. **Golden vectors are immutable**  
   既存 Golden の期待 hash を「都合で」書き換えない。  
   新仕様は新 version + 新 Golden。

4. **payload version ≠ implementation version**  
   crate / package の版と、hash に入る payload version を混同しない。

---

## 4. Security Policy

1. **No hidden mutable state**  
   プロトコルが認める状態以外を、暗黙に持ち越したり共有したりしない。

2. **Hash verification required**  
   Capsule / Seal を受け入れる前に hash を再計算して検証する。

3. **Observer isolation**  
   Observer（Monitor）は推論・品質評価をしない。  
   契約（HashA）遵守と状態差分の観測に限定する。

4. **No silent history rewrite**  
   過去の Seal・承認済み HashB を削除・改変して「なかったこと」にしない。

---

## 5. Runtime Policy

1. **Projection must not modify Canonical State**  
   Projector は入力から新しい Canonical を**生成**する。  
   既存の承認済み Canonical をその場で書き換えてはならない。

2. **Monitor may request review but never rewrite history**  
   Monitor / Observer は Continue / AskUser / Abort / Revise を返せる。  
   履歴そのものを改ざんする権限はない。

3. **Round Reset keeps only approved snapshots**  
   マルチエージェント実行では、持ち越しは HashA・承認済み HashB・Seal・round・roles に限定する。

4. **Framework before Model**  
   Runtime は特定ベンダー LLM に依存しない。Provider は差し替え可能である。

---

## 6. Testing Policy

1. **Every protocol change requires Golden tests**  
   シリアライズ・hash・I/O 契約の変更は Golden で固定する。

2. **Cross-language verification required**  
   少なくとも Python と Rust（または同等の第2実装）で同一 hash を確認する。

3. **Deterministic replay required**  
   同一シード・同一入力で結果を再現できること。非決定論は明示し、hash 境界の外に置く。

4. **Conformance over anecdote**  
   「動いた」ではなく、チェックリストと数値（PASS 数、divergence 等）で記録する。

---

## 7. Documentation Policy

1. **RFC before implementation**（新規プロトコル・破壊的変更）  
   仕様を先に書き、実装はその後。緊急の研究プロトタイプは Research 線に隔離する。

2. **Production and Research are separated**  
   ドキュメント上も `docs/plp-r/` 等で研究線を分け、Production の手順書と混線させない。

3. **Policies live in version control**  
   本 POLICY.md の変更はコミットされ、日付と理由が残る。

4. **Design rationale is written down**  
   「なぜ Dual Hash か」「なぜ Raw を canonical に入れないか」など、意図を文書化する。

---

## Enforcement（運用）

| 層 | 役割 |
|----|------|
| **POLICY.md** | 憲法（本ファイル） |
| **RFC / ROADMAP** | 仕様と優先順位 |
| **Golden / Conformance** | 機械的な拘束 |
| **Code review** | 人による確認 |

衝突時の優先順位:

```
POLICY  >  released Golden / payload contract  >  ROADMAP  >  実装の都合
```

---

## Change Log

| Date | Change |
|------|--------|
| 2026-08-11 | Initial constitutional POLICY.md |

---

*History is Truth · Canonical is Immutable · Difference is Observable · Projection is Replaceable · Framework before Model*

# ACP v2.0 + Capsule v2.0 — Design Sketch

**Date**: 2026-08-09  
**Depends on**: `AXIOM_V2_PRIORITY_AND_DESIGN.md`, PLP-R Phase 1–3

---

## Capsule v2.0 構造（A / B 境界）

```
┌─────────────────────────────────────────────┐
│ Capsule v2.0                                │
│                                             │
│  Header (frozen fields for hash)            │
│    protocol, version, capsule_id,           │
│    clock, sequence, timestamp_ns, source    │
│                                             │
│  ┌─ A-layer (Immutable) ─────────────────┐  │
│  │  raw_text                              │  │
│  │  raw_hash = H(domain=raw ‖ raw_text)   │  │
│  └────────────────────────────────────────┘  │
│                                             │
│  ┌─ B-layer (Projected) ─────────────────┐  │
│  │  CanonicalState                        │  │
│  │    language, tokens, annotations, meta │  │
│  │  canonical_hash = H(domain=canonical   │  │
│  │                     ‖ header ‖ state)  │  │
│  └────────────────────────────────────────┘  │
│                                             │
│  Seal (ACP)                                 │
│    proof = ACP.seal(header, A, B)           │
└─────────────────────────────────────────────┘
```

### 不変ルール

| 層 | 変更 | 誰が読む | 誰が比較する |
|----|------|----------|--------------|
| A (Raw) | 入力後は不変 | LLM | 改ざん検知のみ |
| B (Canonical) | Projector のみが生成 | Agent / DCK | **主比較対象** |
| Seal | ACP のみが付与 | 検証器 | 完全性 |

Annotation は常に **Projection Candidate**。Semantic Truth と宣言しない。

---

## ACP v2.0 ハッシュ仕様（骨子）

### Domain Separation

```
H_raw        = SHA256( "axiom:v2:raw\0"        ‖ utf8(raw_text) )
H_canonical  = SHA256( "axiom:v2:canonical\0"  ‖ canonical_payload )
H_proof      = SHA256( "axiom:v2:proof\0"      ‖ header ‖ H_raw ‖ H_canonical )
```

- `\0` 区切りで domain をバイト列に固定
- canonical_payload は PLP-R の決定論的シリアライザを継承（ソート済み annotations、timestamp_ns は十進文字列）

### HashA / HashB 対応

| DCK dual_hash | Capsule / ACP |
|---------------|---------------|
| HashA (Invariant) | `H_raw` |
| HashB (Semantic/Projected) | `H_canonical` |

分類は既存の DualHashClass をそのまま使用:

| A same | B same | Class |
|--------|--------|-------|
| ✓ | ✓ | None |
| ✓ | ✗ | Semantic |
| ✗ | ✓ | State |
| ✗ | ✗ | Compound |

---

## Proof Chain（最小）

```
1. build CanonicalState          (PLP)
2. compute H_raw, H_canonical    (Capsule)
3. proof = H_proof(header, H_raw, H_canonical)
4. SealedCapsule = Capsule + proof
5. verify: recompute hashes == sealed values
```

Proof Chain の最適化ポイント:

- header の hash 対象フィールドを **凍結リスト** で管理
- 可変メタ（debug 用）は hash 外
- 再計算は pure / sync（PLP-R と同様）

---

## 差分生成（高速パス）

```
diff = diff_canonical(B_a, B_b)     # annotations only
metrics = DifferenceMetrics(diff)
dual = DualHashEvaluation(H_raw_a, H_canonical_a, H_raw_b, H_canonical_b)
monitor = monitor_decide(metrics, integrity_ok)
```

Raw テキストの diff は **行わない**（コスト・ノイズ）。  
改ざんは H_raw 不一致で検知。

---

## バージョン互換

| フィールド | 役割 |
|------------|------|
| `protocol` | `"AXIOM-CAPSULE/2.0"` / `"AXIOM-ACP/2.0"` |
| `payload_version` | hash に入る（変更で Golden 再ロック） |
| `impl_version` | hash に入らない（crate / package 版） |

PLP-R の教訓: payload version を不用意に上げると Golden が全壊する。  
研究リビジョンと payload version を分離する。

---

## 次の具体的成果物

1. **RFC-AXIOM-ACP-2.0** — domain tags, proof format, compatibility
2. **RFC-AXIOM-CAPSULE-2.0** — A/B types, header freeze list
3. **Golden Vector v2 草案** — PLP-R 4本 + ACP seal 列
4. **types/v2** — Python + Rust 共有スキーマ

---

*実験は忠実に実際行って*

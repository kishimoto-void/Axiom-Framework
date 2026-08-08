# DCK Rust v2.3 Improvement Notes

**Date**: 2026-08-08  
**Scope**: Dual-hash difference taxonomy (Hash A / Hash B) + Constraint layer

## Model

```
PLP
  ↓
Hash A (Invariant / Physical)   ← ACP, Ground Truth, Constraint(A)
  │
  ▼
Code / LRP / AI
  │
  ▼
Hash B (Semantic)               ← mutable interpretation
  │
  ▼
Validate(B | Constraint(A)) → Pass | Fail
```

## Judgment matrix

| A     | B     | DualHashClass |
|-------|-------|---------------|
| same  | same  | None          |
| same  | diff  | Semantic      |
| diff  | same  | State         |
| diff  | diff  | Compound      |

## Three difference kinds for DCK

1. **State Difference** — Hash A changed (physical / capsule identity)
2. **Semantic Difference** — Hash B changed under same A (interpretation)
3. **Constraint Difference** — B fails Validate against Constraint(A) (hallucination / invalid)

Constraint Fail takes priority as `primary_kind` even when A and B hashes match.

## New API (`dual_hash.rs`)

```
HashA / HashB
DualHashSnapshot
DualHashClass { None, Semantic, State, Compound }
ConstraintVerdict { Pass, Fail, NotEvaluated }
DifferenceKind { None, State, Semantic, Constraint, Compound }
DualHashEvaluation::evaluate(baseline, current, constraint_ok)
validate_constraint(constraint_id, b_payload, predicate)
```

## Design effect on AXIOM

| Layer | Role |
|-------|------|
| PSS | constraints → feed Constraint(A) |
| PLP / Capsule | Hash A material |
| ACP | seals / proves Hash A |
| Code / LRP / AI | produces Hash B |
| DCK | classifies State / Semantic / Constraint |

A is the only Ground Truth. B may be regenerated freely; validity is membership under Constraint(A).

## Unchanged

- Numeric measurement path (DifferenceMetrics / evaluate_difference)
- Async kernel tick / lease path

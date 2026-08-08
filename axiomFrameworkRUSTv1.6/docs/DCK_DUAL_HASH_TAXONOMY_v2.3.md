# DCK Dual-Hash Taxonomy v2.3

**AXIOM Framework** — design lock for difference classification

---

## 1. Pipeline placement

```
PSS  →  PLP  →  Capsule  →  ACP  →  Hash A (Invariant)
                                      │
                                      │ Constraint(A)
                                      ▼
                              Code / LRP / AI
                                      │
                                      ▼
                                   Hash B (Semantic)
                                      │
                                      ▼
                         DCK.DualHashEvaluation
                         + evaluate_difference (numeric)
```

---

## 2. Roles

| Symbol | Name | Properties |
|--------|------|------------|
| **Hash A** | Invariant / Physical | Sealed by ACP; immutable; Ground Truth |
| **Hash B** | Semantic | Mutable; language / model dependent; regenerable |
| **Constraint(A)** | Constraints attached to A | From PSS (length, format, forbid lists, …) |
| **Validate(B)** | Check B under Constraint(A) | Pass → valid interpretation; Fail → error / hallucination |

---

## 3. Judgment matrix (A × B)

| A \\ B | **same** | **changed** |
|-------|----------|-------------|
| **same** | None | **Semantic Difference** |
| **changed** | **State Difference** | **Compound Difference** |

---

## 4. Three kinds DCK distinguishes

1. **State Difference** — Hash A changed (physical / capsule / input identity)
2. **Semantic Difference** — Hash B changed while A fixed (interpretation drift)
3. **Constraint Difference** — `Validate(B | Constraint(A)) = Fail`  
   (orthogonal; can fire even when A and B hashes match if constraint metadata changes or validation fails)

**Priority**: if constraint fails → `primary_kind = Constraint`.

---

## 5. Why this fits AXIOM

- **ACP** proves A → A is the only Ground Truth.
- **B** may be rewritten by any model; legitimacy is membership under Constraint(A).
- **DCK** is not only numeric residual measurement, but a **classifier** of difference *kind*.
- Hallucination = Semantic or Constraint difference under stable A.

---

## 6. API surface (Rust `dck_modular` 2.3)

```rust
HashA / HashB
DualHashSnapshot { hash_a, hash_b, constraint_id? }
DualHashClass { None, Semantic, State, Compound }
ConstraintVerdict { Pass, Fail, NotEvaluated }
DifferenceKind { None, State, Semantic, Constraint, Compound }
DualHashEvaluation::evaluate(baseline, current, constraint_ok: Option<bool>)
validate_constraint(constraint_id, b_payload, predicate)
```

---

## 7. Example

```rust
// Same physical state, new interpretation that violates length constraint
let base = DualHashSnapshot::new(HashA::new("capsule_abc"), HashB::new("out_v1"));
let cur  = DualHashSnapshot::new(HashA::new("capsule_abc"), HashB::new("out_v2_too_long"));
let ok = Some(false); // Validate failed
let ev = DualHashEvaluation::evaluate(&base, &cur, ok);
assert_eq!(ev.class, DualHashClass::Semantic);
assert_eq!(ev.primary_kind, DifferenceKind::Constraint);
```

---

*Locked with DCK v2.3 — 2026-08-08*

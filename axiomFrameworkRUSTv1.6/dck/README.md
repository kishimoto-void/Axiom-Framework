# Difference Convergence Kernel (DCK) — Rust v2.3

**Part of**: AXIOM Framework Rust v1.6  
**Role**: Runtime kernel + **measurement library** for difference & convergence (numeric + dual-hash taxonomy).

## v2.3 Dual-hash taxonomy

```
Hash A (Invariant) ──ACP──► Ground Truth + Constraint(A)
Hash B (Semantic)  ──AI──► interpretation (mutable)
```

| A \\ B | same | changed |
|-------|------|---------|
| **same** | None | **Semantic** |
| **changed** | **State** | **Compound** |

+ **Constraint Difference** when `Validate(B | Constraint(A))` fails (hallucination / invalid).

```rust
use dck_modular::{
    DualHashEvaluation, DualHashSnapshot, HashA, HashB, DifferenceKind,
};

let base = DualHashSnapshot::new(HashA::new("a1"), HashB::new("b1"));
let cur  = DualHashSnapshot::new(HashA::new("a1"), HashB::new("b2"));
let ev = DualHashEvaluation::evaluate(&base, &cur, Some(true));
assert_eq!(ev.primary_kind, DifferenceKind::Semantic);
```

## v2.2 Highlights (measurement completeness)

| Feature | Type / API | Purpose |
|---------|------------|---------|
| **DifferenceBreakdown** | position, velocity, covariance, confidence | Why residual remains |
| **History / curves** | report.history(), difference_curve() | Plot / Golden / CI |
| **ConvergenceReason** | ThresholdReached / MaxTick / Divergence / … | Why the run stopped |
| **StabilityScore** | score, speed, smoothness, final_accuracy | Paper / README |
| **evaluate_difference** | pure sync | Golden Vector & E2E |

## Layout

```
dck/
├── Cargo.toml          # v2.3.0
├── README.md
├── IMPROVEMENT_NOTES_v2.3.md
└── src/
    ├── dual_hash.rs    # HashA/B, DualHashEvaluation
    ├── metrics.rs      # numeric measurement
    ├── golden_tests.rs
    ├── clock.rs
    ├── state.rs
    └── …
```

## Docs

- `docs/DCK_DUAL_HASH_TAXONOMY_v2.3.md` — design lock
- `golden/dck/*.json` — numeric golden vectors

*DCK v2.3 — dual-hash taxonomy + measurement library for AXIOM Framework*

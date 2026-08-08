# AXIOM Framework — Rust v1.6

Quality-assured pipeline: **PSS → LRP → PLP Capsule → ACP → DCK**

Cross-language determinism (Rust / Python), Golden Vectors, dual-hash difference taxonomy.

---

## Status snapshot (2026-08-08)

| Component | Version / state | Notes |
|-----------|-----------------|--------|
| **DCK** | **v2.3.0** (`dck_modular`) | Full source on tree: dual-hash + metrics + kernel |
| Dual-hash | Hash A / Hash B | State · Semantic · Constraint classification |
| Numeric DCK | `evaluate_difference` | Golden converge / diverge vectors locked |
| PLP Capsule | v1.1.3 (from v1.5) | SHA-256 Golden Hash cross-lang lock |
| ACP | v1.1.0 templates | Integrity / Hash A seal |
| PSS | modular design | types / spec / validation path |
| Docs | Dual-hash taxonomy locked | `docs/DCK_DUAL_HASH_TAXONOMY_v2.3.md` |

---

## Pipeline

```
Input
  │
  ▼
PSS            Problem / constraint normalization
  │
  ▼
LRP            Representation
  │
  ▼
PLP Capsule    Deterministic container → material for Hash A
  │
  ▼
ACP            Seals Hash A (Invariant / Ground Truth) + Constraint(A)
  │
  ▼
Code / AI      Produces Hash B (Semantic interpretation)
  │
  ▼
DCK            DualHashEvaluation + evaluate_difference
  │
  ▼
Difference kind + numeric residual + convergence report
```

### DCK judgment (v2.3)

| A \\ B | same | changed |
|-------|------|---------|
| **same** | None | **Semantic** |
| **changed** | **State** | **Compound** |

+ **Constraint Difference** if `Validate(B | Constraint(A))` fails.

---

## Layout

```
axiomFrameworkRUSTv1.6/
├── README.md                          ← this file
├── dck/                               ★ complete crate (v2.3.0)
│   ├── Cargo.toml
│   ├── README.md
│   ├── IMPROVEMENT_NOTES_v2.{1,2,3}.md
│   └── src/
│       ├── dual_hash.rs               HashA/B, DualHashEvaluation
│       ├── metrics.rs                 DifferenceMetrics, evaluate_difference
│       ├── kernel.rs                  DifferenceConvergenceKernel
│       ├── clock.rs / state.rs / …
│       └── golden_tests.rs
├── docs/
│   └── DCK_DUAL_HASH_TAXONOMY_v2.3.md
├── golden/
│   └── dck/
│       ├── dck_scalar_converge_01.json
│       └── dck_scalar_diverge_01.json
├── pss/  acp/  plp_capsule/  lrp/     (v1.5 cores / ongoing)
├── scripts/
└── integration/
```

Remote:  
https://github.com/kishimoto-void/Axiom-Framework/tree/main/axiomFrameworkRUSTv1.6

---

## Quick start — DCK

```bash
cd axiomFrameworkRUSTv1.6/dck
cargo test --lib
```

```rust
use dck_modular::{
    DualHashEvaluation, DualHashSnapshot, HashA, HashB,
    evaluate_difference, DCKConfig, DifferenceKind,
};

// Dual-hash classification
let base = DualHashSnapshot::new(HashA::new("a1"), HashB::new("b1"));
let cur  = DualHashSnapshot::new(HashA::new("a1"), HashB::new("b2"));
let ev = DualHashEvaluation::evaluate(&base, &cur, Some(true));
assert_eq!(ev.primary_kind, DifferenceKind::Semantic);

// Numeric residual
let metrics = evaluate_difference(&estimate, &target, &DCKConfig::default())?;
```

---

## Design principles

1. **Prove, don’t claim** — Golden Vectors + deterministic clocks  
2. **A is Ground Truth** — ACP seals Hash A; B is always regenerable  
3. **Difference has kind and magnitude** — taxonomy + Mahalanobis residual  
4. **Measurement ≠ Runtime** — pure sync metrics for CI; async tick for execution  
5. **Cross-language lock** — PLP / metrics JSON comparable Rust ↔ Python  

---

## Related documents

| Doc | Role |
|-----|------|
| `dck/README.md` | DCK crate detail |
| `dck/IMPROVEMENT_NOTES_v2.3.md` | Dual-hash change log |
| `docs/DCK_DUAL_HASH_TAXONOMY_v2.3.md` | Design lock |
| `golden/dck/*.json` | Numeric golden vectors |

---

## Roadmap after v1.6 / DCK 2.3

1. Wire PSS → Capsule → ACP → DCK dual-hash in one E2E integration test  
2. Fill remaining ACP Golden numbers from locked reference runs  
3. Enable GitHub Actions on every PR (crate test + golden verify)  
4. Python mirror of `DualHashEvaluation` for CI parity  

When CI is green on every PR, AXIOM moves from research prototype to **continuously quality-assured framework**.

---

*AXIOM Framework Rust v1.6 — DCK v2.3 complete on main (2026-08-08)*

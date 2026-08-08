# Difference Convergence Kernel (DCK) — Rust v2.3

**Package**: `dck_modular` **2.3.0**  
**Part of**: [AXIOM Framework Rust v1.6](../)  
**Role**: Runtime kernel **and** measurement library — numeric residual + dual-hash difference taxonomy

---

## Status (2026-08-08)

| Layer | State |
|-------|--------|
| Source tree | Complete under `src/` (18 modules) |
| Dual-hash taxonomy | **v2.3** — `dual_hash.rs` |
| Numeric measurement | **v2.2 API** — `metrics.rs` |
| Runtime kernel | `kernel.rs` + `KernelBuilder` |
| Determinism | `MockClock` |
| Golden vectors | `../golden/dck/*.json` |
| Design lock | `../docs/DCK_DUAL_HASH_TAXONOMY_v2.3.md` |

---

## Pipeline position

```
PSS → PLP → Capsule → ACP → Hash A (Invariant / Ground Truth)
                              │ Constraint(A)
                              ▼
                        Code / LRP / AI
                              ▼
                           Hash B (Semantic)
                              ▼
              DCK ── DualHashEvaluation  +  evaluate_difference
```

---

## v2.3 Dual-hash taxonomy

Hash **A** is sealed by ACP and treated as Ground Truth.  
Hash **B** is a regenerable interpretation (model / human).

| A \\ B | same | changed |
|-------|------|---------|
| **same** | None | **Semantic** |
| **changed** | **State** | **Compound** |

+ **Constraint Difference** when `Validate(B | Constraint(A))` fails  
  (hallucination / invalid interpretation; can fire even if hashes match)

```rust
use dck_modular::{
    DualHashEvaluation, DualHashSnapshot, HashA, HashB, DifferenceKind,
};

let base = DualHashSnapshot::new(HashA::new("capsule_abc"), HashB::new("out_v1"));
let cur  = DualHashSnapshot::new(HashA::new("capsule_abc"), HashB::new("out_v2"));
let ev = DualHashEvaluation::evaluate(&base, &cur, Some(true));
assert_eq!(ev.primary_kind, DifferenceKind::Semantic);
```

Three kinds DCK classifies:

1. **State** — Hash A changed  
2. **Semantic** — Hash B changed under fixed A  
3. **Constraint** — B fails validation against Constraint(A)

---

## v2.2 Measurement API (numeric)

| API | Purpose |
|-----|---------|
| `DifferenceMetrics` | `difference_total`, `convergence_rate`, `converged`, `breakdown` |
| `DifferenceBreakdown` | `position`, `velocity`, `covariance`, `confidence` |
| `ConvergenceReport` | history, curves, `ticks_to_threshold`, `finish` |
| `ConvergenceReason` | ThresholdReached / MaxTick / Divergence / NumericalIssue / InProgress |
| `StabilityScore` | 0..1 composite (speed · smoothness · final_accuracy) |
| `evaluate_difference` | pure sync path for Golden / CI / E2E |

```rust
use dck_modular::{evaluate_difference, ConvergenceReport, DCKConfig, StateEstimate};

let metrics = evaluate_difference(&estimate, &target, &config)?;
// metrics.difference_total, convergence_rate, breakdown, converged

let mut report = ConvergenceReport::new();
report.push(metrics, &config);
report.finish(max_ticks, &config);
// report.reason, report.difference_curve(), report.stability
```

JSON-friendly metrics contract (serde):

```json
{
  "difference_total": 0.021,
  "convergence_rate": 0.983,
  "converged": true,
  "dim": 1,
  "breakdown": {
    "position": 0.021,
    "velocity": 0.0,
    "covariance": 0.01,
    "confidence": 0.05
  }
}
```

---

## Layout

```
dck/
├── Cargo.toml                 # dck_modular 2.3.0
├── README.md
├── IMPROVEMENT_NOTES_v2.1.md
├── IMPROVEMENT_NOTES_v2.2.md
├── IMPROVEMENT_NOTES_v2.3.md
└── src/
    ├── lib.rs                 # exports
    ├── dual_hash.rs           # HashA/B, DualHashEvaluation (v2.3)
    ├── metrics.rs             # numeric measurement (v2.2)
    ├── golden_tests.rs
    ├── clock.rs               # SystemClock + MockClock
    ├── state.rs               # StateEstimate + Mahalanobis
    ├── kernel.rs              # DifferenceConvergenceKernel
    ├── config.rs / error.rs / event.rs / gap.rs / ids.rs
    ├── intent.rs / lease.rs / resource.rs
    ├── capabilities.rs / stubs.rs
    └── main.rs                # demo binary
```

Related (sibling paths under `axiomFrameworkRUSTv1.6/`):

- `docs/DCK_DUAL_HASH_TAXONOMY_v2.3.md` — design lock  
- `golden/dck/dck_scalar_converge_01.json`  
- `golden/dck/dck_scalar_diverge_01.json`

---

## Build & test

```bash
cd axiomFrameworkRUSTv1.6/dck
cargo test --lib
cargo run --release   # optional demo tick
```

Golden numerical check (from v1.6 root, when script present):

```bash
python scripts/verify_dck_numerical_golden.py
```

---

## Version history

| Ver | Focus |
|-----|--------|
| **2.3** | Dual-hash taxonomy (State / Semantic / Constraint) |
| **2.2** | Measurement completeness (Breakdown, History, Reason, Stability) |
| **2.1** | Metrics API + MockClock + Golden unit tests |
| **2.0** | Modular kernel + nalgebra multi-D |

---

## Design principles

- **A is Ground Truth** — ACP seals Hash A; B is always reinterpretable  
- **Difference has kind** — not only magnitude (`evaluate_difference`) but class (`DualHashEvaluation`)  
- **Measurement ≠ Runtime** — pure sync metrics for CI; async `tick` for execution  
- **Determinism** — `MockClock` so Golden Vectors stay stable across years  

*DCK v2.3 — dual-hash taxonomy + measurement library for AXIOM Framework*

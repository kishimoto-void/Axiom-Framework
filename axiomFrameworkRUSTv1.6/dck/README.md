# Difference Convergence Kernel (DCK) — Rust v2.2

**Part of**: AXIOM Framework Rust v1.6  
**Role**: Runtime kernel + **measurement library** for difference & convergence.

## v2.2 Highlights

| Feature | API |
|---------|-----|
| DifferenceBreakdown | position, velocity, covariance, confidence |
| History / curves | report.history(), difference_curve(), convergence_curve() |
| ConvergenceReason | ThresholdReached / MaxTick / Divergence / NumericalIssue / InProgress |
| StabilityScore | score, speed, smoothness, final_accuracy (0..1) |
| evaluate_difference | pure sync (Golden / E2E / CI) |
| MockClock | deterministic time for tests |

## Usage

```rust
use dck_modular::{evaluate_difference, ConvergenceReport, DCKConfig, StateEstimate};

let metrics = evaluate_difference(&estimate, &target, &config)?;
// metrics.difference_total, convergence_rate, breakdown, converged

let mut report = ConvergenceReport::new();
report.push(metrics, &config);
report.finish(max_ticks, &config);
// report.reason, report.difference_curve(), report.stability
```

## Layout

```
dck/
├── Cargo.toml          # v2.2.0
├── README.md
├── IMPROVEMENT_NOTES_v2.2.md
└── src/
    ├── metrics.rs      # measurement contract
    ├── golden_tests.rs
    ├── clock.rs        # MockClock
    ├── state.rs        # nalgebra StateEstimate
    ├── kernel.rs       # async tick path
    └── ...
```

## Tests

```bash
cd axiomFrameworkRUSTv1.6/dck
cargo test --lib
```

## Golden vectors

See `../golden/dck/*.json` (converge / diverge locked 2026-08-08).

*DCK v2.2 — measurement library for AXIOM Framework*

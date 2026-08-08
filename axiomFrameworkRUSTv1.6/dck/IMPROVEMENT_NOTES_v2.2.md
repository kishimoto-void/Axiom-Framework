# DCK Rust v2.2 Improvement Notes

**Date**: 2026-08-08  
**Scope**: Rust DCK measurement layer

## Requested → Delivered

| Request | Implementation |
|---------|----------------|
| ① Difference Breakdown | `DifferenceBreakdown { position, velocity, covariance, confidence }` |
| ② History | `ConvergenceReport::history()`, `difference_curve()`, `convergence_curve()` |
| ③ Convergence Reason | `ConvergenceReason { ThresholdReached, MaxTick, Divergence, NumericalIssue, InProgress }` |
| ④ Stability Score | `StabilityScore { score, speed, smoothness, final_accuracy }` (0..1) |

## API surface (public)

```
DifferenceBreakdown
DifferenceMetrics { …, breakdown }
ConvergenceReason
StabilityScore
ConvergenceReport {
  push / finish / mark_numerical_issue
  history / difference_curve / convergence_curve
  reason / stability / ticks_to_threshold
}
evaluate_difference
evaluate_difference_with_velocity
```

## Behaviour notes

- **position**: Σ |mean_i − target_i|
- **velocity**: |smoothed velocity| when provided via `evaluate_difference_with_velocity`
- **covariance**: `StateEstimate::total_uncertainty`
- **confidence**: `(1 − confidence) * weight_risk`
- **Divergence**: difference grows >2× previous and above 10× tolerance
- **StabilityScore**: 0.35·speed + 0.30·smoothness + 0.35·final_accuracy

## Unchanged

- Async `tick` / lease / executor path (v2.0 design)
- Resource / Intent / Capability traits

## Next optional steps

1. Fill `golden/dck/*.json` expected fields from measured v2.2 runs
2. Emit `ConvergenceReport` from kernel after N ticks
3. Python mirror of Breakdown / Reason / Stability for cross-lang CI

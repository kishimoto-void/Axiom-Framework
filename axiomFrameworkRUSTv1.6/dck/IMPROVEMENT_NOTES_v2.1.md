# DCK Rust v2.1 Improvement Notes

**Date**: 2026-08-07  
**Scope**: Rust DCK only (as requested)

## Goals addressed

1. **Numerical Golden Vector readiness**
   - Explicit `DifferenceMetrics { difference_total, convergence_rate, converged, per_dim, dim }`
   - `ConvergenceReport { ticks, ticks_to_threshold, final_*, history }`
   - Pure `evaluate_difference` (sync, no tokio)

2. **Determinism for tests**
   - `MockClock` with fixed base + `advance_ms`

3. **Locked unit behaviour**
   - On-target ≈ 0 difference
   - Far > near difference
   - 2-D Mahalanobis respects covariance axis
   - Monotonic improvement sequence

4. **CI / serde**
   - Metrics derive Serialize/Deserialize for dump/compare

## What was intentionally not changed

- Async `tick` / lease / executor path (stable v2.0 design)
- Resource / Intent / Capability traits
- Overall module split

## Next steps

1. Run `cargo test --lib` on a normal toolchain and capture measured Golden numbers
2. Fill `golden/dck/*.json` expected fields from those measurements
3. Wire `evaluate_difference` into the integrated E2E test
4. Optional: add a small binary that reads a Golden JSON and asserts metrics within tolerance

## Files touched

| File | Change |
|------|--------|
| `src/metrics.rs` | **New** |
| `src/golden_tests.rs` | **New** |
| `src/clock.rs` | + MockClock |
| `src/lib.rs` | exports + version note |
| `Cargo.toml` | 2.1.0 + serde |
| `README.md` | v2.1 docs |

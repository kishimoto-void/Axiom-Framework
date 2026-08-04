# DCK Modular (Rust) v2.0

Difference Convergence Kernel — fully modular + nalgebra multi-dimensional implementation.

Part of **Axiom-Framework** / `axiomFrameworkRUSTv1.5`.

## Features

- Strict Newtype IDs (no blanket `From`)
- Real multi-dimensional state via `nalgebra` (`DVector` / `DMatrix` + Cholesky)
- 1-D convenience path (`StateEstimate::scalar`)
- Injected `Clock` for testability
- Fine-grained locking + `futures::join_all` concurrency
- Semaphore-limited parallel execution
- Config-driven scoring weights and numerical floors

## Structure

```
dck/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── main.rs
    ├── clock.rs
    ├── ids.rs
    ├── error.rs
    ├── config.rs
    ├── state.rs      # nalgebra core
    ├── resource.rs
    ├── lease.rs
    ├── capabilities.rs
    ├── intent.rs
    ├── event.rs
    ├── gap.rs
    ├── kernel.rs
    └── stubs.rs
```

## Build

```bash
cd axiomFrameworkRUSTv1.5/dck
cargo run
```

## Relation to Python DCK

The Python implementation lives under `src/modules/dck/`.
This Rust version is the performance / type-safety oriented counterpart with proper multi-D linear algebra.

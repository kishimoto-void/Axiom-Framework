# axiomFrameworkgo

Go implementation of MiniAXIOM / Axiom Framework components.

## Packages

- `pkg/capsule` — Deterministic Capsule with Clock, BasisPoint, SectionGate, invariants
- `pkg/adapter` — LLM adapter pipeline (Markdown / XML renderers, Registry, Fuzz & Golden)
- `pkg/dck` — Difference Convergence Kernel (ring-buffer history, O(1) Evaluate)
- `pkg/pss` — Predictive State Specification DSL parser / formatter

## Status

- Golden Vector tests included (SHA-256 verified)
- Race-free (`go test -race`)
- Determinism verified (10k+ runs)
- Fuzz tested

## Module

```
module miniaxiom

go 1.22
```

Run tests:

```bash
cd axiomFrameworkgo
go test ./...
go test -race ./...
go test -bench=. -benchmem ./...
```

## License

Same as parent Axiom-Framework repository.

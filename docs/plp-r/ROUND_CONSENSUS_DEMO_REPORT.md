# Round Consensus Protocol — Minimal Demo Report

**Date**: 2026-08-09 13:24:52 UTC  
**Status**: PASS  
**Checks**: 10/10  
**Protocol**: Round Consensus v0.1  

---

## Setup

- Agents: `['α', 'β', 'γ']`
- Specialty: `{'α': 'process', 'β': 'tooling', 'γ': 'culture'}`
- HashA: `fd494a56f32c94ab8d0f45cc…`

HashA contract (summary): code-review quality next steps; no secrets; Observer = contract only.

---

## Round 0

| Agent | Role |
|-------|------|
| α | **Observer** |
| β | **Reasoner** |
| γ | **Reasoner** |

**Observer**: `α`  
**Reasoners**: `['β', 'γ']`  

### HashB candidates

| Agent | Mode | Title | Tags | hash_b |
|-------|------|-------|------|--------|
| β | in_scope | `linter-bot` | ['ACTION(enable_bot)', 'METRIC(false_positive_rate)'] | `5b92d85a83e5…` |
| γ | in_scope | `review-sla` | ['ACTION(set_sla)', 'METRIC(median_response_time)'] | `15d68ae31563…` |

### ObserverVerdict

```json
{
  "kind": "Accept",
  "summary": "All candidates within HashA scope; no safety violations",
  "candidates": [
    "linter-bot",
    "review-sla"
  ]
}
```

**Sealed**: `True`  

### Seal

- proof: `7c235f016a6b58ed28340b63…`
- approved: `['linter-bot', 'review-sla']`
- roles: `{'α': 'Observer', 'β': 'Reasoner', 'γ': 'Reasoner'}`

### Carried forward (after Reset)

```json
{
  "round": 0,
  "hash_a": "fd494a56f32c94ab…",
  "approved_titles": [
    "linter-bot",
    "review-sla"
  ],
  "seal_count": 1,
  "roles": {
    "α": "Observer",
    "β": "Reasoner",
    "γ": "Reasoner"
  }
}
```

---

## Round 1

| Agent | Role |
|-------|------|
| α | **Reasoner** |
| β | **Observer** |
| γ | **Reasoner** |

**Observer**: `β`  
**Reasoners**: `['α', 'γ']`  

### HashB candidates

| Agent | Mode | Title | Tags | hash_b |
|-------|------|-------|------|--------|
| α | soft_violation | `rewrite-org` | ['ACTION(reorg)', 'SCOPE(org_wide)'] | `807aeecd5a70…` |
| γ | in_scope | `review-sla` | ['ACTION(set_sla)', 'METRIC(median_response_time)'] | `15d68ae31563…` |

### ObserverVerdict

```json
{
  "kind": "Revise",
  "reason": "Goal drift (org-wide scope) by α: rewrite-org",
  "summary": "Stay within concrete next steps for code review quality",
  "candidates": [
    "rewrite-org",
    "review-sla"
  ]
}
```

**Sealed**: `False`  

### Carried forward (after Reset)

```json
{
  "round": 1,
  "hash_a": "fd494a56f32c94ab…",
  "approved_titles": [
    "linter-bot",
    "review-sla"
  ],
  "seal_count": 1,
  "roles": {
    "α": "Reasoner",
    "β": "Observer",
    "γ": "Reasoner"
  }
}
```

### Divergence vs previous approved

- divergence: **0.0**
- added: `[]`
- removed: `[]`
- overlap: `['ACTION(enable_bot)', 'ACTION(set_sla)', 'METRIC(false_positive_rate)', 'METRIC(median_response_time)']`

---

## Final state

```json
{
  "round": 1,
  "hash_a": "fd494a56f32c94ab…",
  "approved_titles": [
    "linter-bot",
    "review-sla"
  ],
  "seal_count": 1,
  "roles": {
    "α": "Reasoner",
    "β": "Observer",
    "γ": "Reasoner"
  }
}
```

## Check List

| ID | Expect | Got | Pass |
|----|--------|-----|------|
| `R0-observer-is-alpha` | `α` | `α` | ✅ |
| `R0-verdict-accept` | `Accept` | `Accept` | ✅ |
| `R0-sealed` | `True` | `True` | ✅ |
| `R0-two-reasoners` | `2` | `2` | ✅ |
| `R1-observer-is-beta` | `β` | `β` | ✅ |
| `R1-verdict-revise` | `Revise` | `Revise` | ✅ |
| `R1-not-sealed` | `False` | `False` | ✅ |
| `R1-carries-r0-approval` | `from R0 seal` | `['linter-bot', 'review-sla']` | ✅ |
| `hash-a-stable` | `fd494a56f32c94ab8d0f45cc53b190c36fbab133d35ca73cf12a9059fb649ac2` | `fd494a56f32c94ab8d0f45cc53b190c36fbab133d35ca73cf12a9059fb649ac2` | ✅ |
| `divergence-logged` | `present` | `True` | ✅ |

---

## Observations

1. Clock rotation moved Observer α → β across rounds.
2. Round 0: both Reasoners in-scope → Accept + Seal.
3. Round 1: α as Reasoner emitted org-wide drift → Observer(β) returned **Revise** (contract, not quality).
4. Round Reset kept HashA + prior approved HashB + Seal; rejected candidates were dropped.
5. Divergence was logged without claiming monotonic convergence.

*実験は忠実に実際行って*

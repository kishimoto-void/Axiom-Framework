# PLP-R Phase 3 — Monitor Demo Test Report

**Date**: 2026-08-08 16:11:07 UTC  
**Status**: PASS  
**Checks**: 15/15  

---

## Summary

| Scenario | Turns | Final baseline | Result |
|----------|-------|----------------|--------|
| `scenario_sleep_run_then_confirm` | 3 | `PlanRun` | PASS |
| `scenario_identical_start` | 2 | `CatPlan` | PASS |
| `scenario_triple_agent_style` | 2 | `PlanSleep` | PASS |

**Overall**: 15/15 checks passed

---

## scenario_sleep_run_then_confirm

> sleep vs run → user picks sleep → agree → diverge → user picks run

### Turns

| Turn | A | B | dual | div | monitor | user | baseline before → after |
|------|---|---|------|-----|---------|------|-------------------------|
| 1 | `PlanSleep` | `PlanRun` | Compound | 0.500 | **AskUser** | PlanSleep | ∅ → **PlanSleep** |
| 2 | `PlanSleep` | `PlanSleep-B` | None | 0.000 | **Continue** | — | PlanSleep → **PlanSleep** |
| 3 | `PlanSleep` | `PlanRun` | Compound | 0.500 | **AskUser** | PlanRun | PlanSleep → **PlanRun** |

### Annotation detail

**Turn 1**
- A raw: `猫が机の上で寝ている。`
- B raw: `猫が机の上で走っている。`
- A ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(sleep)']
- B ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(run)']
- kind_status: `{'ENTITY': 'same', 'LOCATION': 'same', 'ACTION': 'mixed'}`
- summary: `[DIVERGE] dual=Compound kind=Compound divergence=0.500 monitor=AskUser`
- monitor_detail: `{'summary': 'Canonical state candidates diverged: ACTION=mixed', 'candidates': ['PlanSleep', 'PlanRun']}`

**Turn 2**
- A raw: `猫が机の上で寝ている。`
- B raw: `猫が机の上で寝ている。`
- A ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(sleep)']
- B ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(sleep)']
- kind_status: `{'ENTITY': 'same', 'LOCATION': 'same', 'ACTION': 'same'}`
- summary: `[SAME] dual=None kind=None divergence=0.000 monitor=Continue`

**Turn 3**
- A raw: `猫が机の上で寝ている。`
- B raw: `猫が机の上で走っている。`
- A ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(sleep)']
- B ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(run)']
- kind_status: `{'ENTITY': 'same', 'LOCATION': 'same', 'ACTION': 'mixed'}`
- summary: `[DIVERGE] dual=Compound kind=Compound divergence=0.500 monitor=AskUser`
- monitor_detail: `{'summary': 'Canonical state candidates diverged: ACTION=mixed', 'candidates': ['PlanSleep', 'PlanRun']}`

### Final baseline

```json
{
  "turn": 3,
  "baseline_label": "PlanRun",
  "baseline_raw": "猫が机の上で走っている。",
  "baseline_annotations": [
    "ENTITY(cat)",
    "LOCATION(table)",
    "ACTION(run)"
  ],
  "history_len": 3
}
```

---

## scenario_identical_start

> identical → Continue; then Cat vs Neutral → AskUser

### Turns

| Turn | A | B | dual | div | monitor | user | baseline before → after |
|------|---|---|------|-----|---------|------|-------------------------|
| 1 | `AgentA` | `AgentB` | None | 0.000 | **Continue** | — | ∅ → **AgentA** |
| 2 | `CatPlan` | `NeutralPlan` | Compound | 1.000 | **AskUser** | CatPlan | AgentA → **CatPlan** |

### Annotation detail

**Turn 1**
- A raw: `cat sleeps on table`
- B raw: `cat sleeps on table`
- A ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(sleep)']
- B ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(sleep)']
- kind_status: `{'ENTITY': 'same', 'LOCATION': 'same', 'ACTION': 'same'}`
- summary: `[SAME] dual=None kind=None divergence=0.000 monitor=Continue`

**Turn 2**
- A raw: `cat sleeps on table`
- B raw: `the sky is blue`
- A ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(sleep)']
- B ann: []
- kind_status: `{'ENTITY': 'removed', 'LOCATION': 'removed', 'ACTION': 'removed'}`
- summary: `[DIVERGE] dual=Compound kind=Compound divergence=1.000 monitor=AskUser`
- monitor_detail: `{'summary': 'Canonical state candidates diverged: ENTITY=removed, LOCATION=removed, ACTION=removed', 'candidates': ['CatPlan', 'NeutralPlan']}`

### Final baseline

```json
{
  "turn": 2,
  "baseline_label": "CatPlan",
  "baseline_raw": "cat sleeps on table",
  "baseline_annotations": [
    "ENTITY(cat)",
    "LOCATION(table)",
    "ACTION(sleep)"
  ],
  "history_len": 2
}
```

---

## scenario_triple_agent_style

> pairwise: sleep/run → sleep; sleep/neutral → keep sleep

### Turns

| Turn | A | B | dual | div | monitor | user | baseline before → after |
|------|---|---|------|-----|---------|------|-------------------------|
| 1 | `PlanSleep` | `PlanRun` | Compound | 0.500 | **AskUser** | PlanSleep | ∅ → **PlanSleep** |
| 2 | `PlanSleep` | `Neutral` | Compound | 1.000 | **AskUser** | PlanSleep | PlanSleep → **PlanSleep** |

### Annotation detail

**Turn 1**
- A raw: `猫が机の上で寝ている。`
- B raw: `猫が机の上で走っている。`
- A ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(sleep)']
- B ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(run)']
- kind_status: `{'ENTITY': 'same', 'LOCATION': 'same', 'ACTION': 'mixed'}`
- summary: `[DIVERGE] dual=Compound kind=Compound divergence=0.500 monitor=AskUser`
- monitor_detail: `{'summary': 'Canonical state candidates diverged: ACTION=mixed', 'candidates': ['PlanSleep', 'PlanRun']}`

**Turn 2**
- A raw: `猫が机の上で寝ている。`
- B raw: `the sky is blue`
- A ann: ['ENTITY(cat)', 'LOCATION(table)', 'ACTION(sleep)']
- B ann: []
- kind_status: `{'ENTITY': 'removed', 'LOCATION': 'removed', 'ACTION': 'removed'}`
- summary: `[DIVERGE] dual=Compound kind=Compound divergence=1.000 monitor=AskUser`
- monitor_detail: `{'summary': 'Canonical state candidates diverged: ENTITY=removed, LOCATION=removed, ACTION=removed', 'candidates': ['PlanSleep', 'Neutral']}`

### Final baseline

```json
{
  "turn": 2,
  "baseline_label": "PlanSleep",
  "baseline_raw": "猫が机の上で寝ている。",
  "baseline_annotations": [
    "ENTITY(cat)",
    "LOCATION(table)",
    "ACTION(sleep)"
  ],
  "history_len": 2
}
```

---

## Check List

| ID | Expect | Got | Pass |
|----|--------|-----|------|
| `S1-T1-decision` | `AskUser` | `AskUser` | ✅ |
| `S1-T1-baseline` | `PlanSleep` | `PlanSleep` | ✅ |
| `S1-T1-dual` | `Compound` | `Compound` | ✅ |
| `S1-T2-decision` | `Continue` | `Continue` | ✅ |
| `S1-T2-divergence` | `0.0` | `0.0` | ✅ |
| `S1-T3-decision` | `AskUser` | `AskUser` | ✅ |
| `S1-T3-choice` | `PlanRun` | `PlanRun` | ✅ |
| `S1-T3-baseline` | `PlanRun` | `PlanRun` | ✅ |
| `S2-T1-decision` | `Continue` | `Continue` | ✅ |
| `S2-T1-dual` | `None` | `None` | ✅ |
| `S2-T2-decision` | `AskUser` | `AskUser` | ✅ |
| `S2-T2-baseline` | `CatPlan` | `CatPlan` | ✅ |
| `S3-T1-baseline` | `PlanSleep` | `PlanSleep` | ✅ |
| `S3-T2-decision` | `AskUser` | `AskUser` | ✅ |
| `S3-T2-baseline-kept` | `PlanSleep` | `PlanSleep` | ✅ |

---

## Design notes

1. Monitor is a pure state machine: baseline + history.
2. AskUser is simulated via `user_policy` (prefer_a / prefer_b / prefer_baseline).
3. Continue keeps or initializes baseline without user input.
4. Dual-hash class and annotation metrics come from Phase 2 bridge.
5. Production PLP Capsule v1.1.3 is untouched.

*実験は忠実に実際行って*

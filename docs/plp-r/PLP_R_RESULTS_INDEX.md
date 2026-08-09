# PLP-R Research Results (Phase 1–3)

**Date**: 2026-08-09  
**Status**: Phase 1–3 complete + Round Consensus minimal demo  
**Line**: Research only (Production PLP Capsule v1.1.3 untouched)

---

## Phase summary

| Phase | Content | Result |
|-------|---------|--------|
| 1 | Golden Vector (cross-language determinism) | ✅ LOCKED |
| 2 | DCK Bridge (DualHash + DifferenceMetrics) | ✅ PASS |
| 3 | Monitor demo (AskUser / baseline state machine) | ✅ **15/15 PASS** |
| RC | Round Consensus (3 agents × 2 rounds) | ✅ **10/10 PASS** |

---

## Test reports

| File | Description |
|------|-------------|
| [PLP_MONITOR_DEMO_TEST_REPORT.md](./PLP_MONITOR_DEMO_TEST_REPORT.md) | Phase 3 Monitor — full turn log + checklist |
| [PLP_R_GOLDEN_VECTORS_v0_1.md](./PLP_R_GOLDEN_VECTORS_v0_1.md) | Phase 1 Golden definitions |
| [PLP_R_GOLDEN_LOCK_v0_1.json](./PLP_R_GOLDEN_LOCK_v0_1.json) | Phase 1 locked dual hashes |
| [PLP_DCK_BRIDGE_DEMO_RESULT.json](./PLP_DCK_BRIDGE_DEMO_RESULT.json) | Phase 2 bridge demo results |
| [PLP_MONITOR_DEMO_RESULT.json](./PLP_MONITOR_DEMO_RESULT.json) | Phase 3 raw results |
| [ROUND_CONSENSUS_DEMO_REPORT.md](./ROUND_CONSENSUS_DEMO_REPORT.md) | 3 agents × 2 rounds minimal demo (10/10 PASS) |
| [ROUND_CONSENSUS_DEMO_RESULT.json](./ROUND_CONSENSUS_DEMO_RESULT.json) | Demo raw result + checks |

## Research notes

| File | Description |
|------|-------------|
| [RESEARCH_PLP_PROJECTION_v0_1.md](./RESEARCH_PLP_PROJECTION_v0_1.md) | PLP-R design contracts |
| [RESEARCH_PLP_DCK_BRIDGE_v0_1.md](./RESEARCH_PLP_DCK_BRIDGE_v0_1.md) | DualHash mapping |
| [RESEARCH_PLP_MONITOR_v0_1.md](./RESEARCH_PLP_MONITOR_v0_1.md) | Monitor state machine |

---

## Key design contracts

1. Annotation = Canonical Projection Candidate (not Semantic Truth)
2. Dual Hash: `raw_hash` (HashA) / `canonical_hash` (HashB)
3. Monitor primary signal = Canonical State divergence (not header noise)
4. Promotion path: PLP-R v0.1 → … → Candidate → PLP v2.0

---

## Related (v2.0 coordination)

| Document | Description |
|----------|-------------|
| [../ROUND_CONSENSUS_PROTOCOL_v0_1.md](../ROUND_CONSENSUS_PROTOCOL_v0_1.md) | Multi-agent Round Consensus under HashA (Observer / Reasoner rotation) |
| [ROUND_CONSENSUS_DEMO_REPORT.md](./ROUND_CONSENSUS_DEMO_REPORT.md) | Minimal demo proving clock rotation + contract-only Observer |

---

*実験は忠実に実際行って*

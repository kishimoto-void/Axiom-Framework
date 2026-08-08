# PLP-R Phase 3 — Monitor Demo Research Note

**Date**: 2026-08-09  
**Status**: ✅ PASS (15/15 checks)  
**Report**: `PLP_MONITOR_DEMO_TEST_REPORT.md`

---

## 目的

マルチエージェント制御の最小形:

```
Capsule A / Capsule B
        ↓
  DualHash + DifferenceMetrics
        ↓
     Monitor
    ╱        ╲
Continue    AskUser → user selects → update baseline
```

---

## 実装上の発見

1. **Monitor の主信号は Canonical State（Annotation）差分**  
   Dual-hash の header 由来ノイズ（capsule_id 差など）だけでは AskUser にしない。

2. **Content-addressed capsule_id**  
   同一 raw → 同一 dual hash。そうしないと「同一内容なのに Semantic」になる。

3. **baseline 状態機械**  
   Turn をまたいで採用された Plan を保持し、次の比較の基準にする。

---

## シナリオ結果

| Scenario | 結果 | Final baseline |
|----------|------|----------------|
| sleep→run 分岐→再分岐 | PASS | PlanRun |
| 同一開始→中立分岐 | PASS | CatPlan |
| pairwise 3エージェント風 | PASS | PlanSleep |

詳細は `PLP_MONITOR_DEMO_TEST_REPORT.md`。

---

## ファイル

| ファイル | 役割 |
|---------|------|
| `plp_monitor_demo.py` | Monitor 状態機械 + 3シナリオ |
| `PLP_MONITOR_DEMO_TEST_REPORT.md` | テスト記録（本レポート） |
| `PLP_MONITOR_DEMO_RESULT.json` | 生データ |

---

## Phase 全体

| Phase | 状態 |
|-------|------|
| 1 Golden | ✅ LOCKED |
| 2 DCK Bridge | ✅ PASS |
| 3 Monitor | ✅ PASS 15/15 |

---

*実験は忠実に実際行って*

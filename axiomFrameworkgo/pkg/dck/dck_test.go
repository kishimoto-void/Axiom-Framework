package dck_test

import (
	"context"
	"fmt"
	"sync"
	"testing"
	"time"

	"miniaxiom/pkg/capsule"
	"miniaxiom/pkg/dck"
)

type MockClock struct {
	mu          sync.Mutex
	currentTime time.Time
}

func NewMockClock(now time.Time) *MockClock {
	return &MockClock{currentTime: now}
}

func (m *MockClock) Now() time.Time {
	m.mu.Lock()
	defer m.mu.Unlock()
	return m.currentTime
}

func (m *MockClock) Advance(d time.Duration) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.currentTime = m.currentTime.Add(d)
}

func setupTestCapsule(id, start, goal string) *capsule.Capsule {
	return &capsule.Capsule{
		ID:         id,
		StartState: start,
		Goal: capsule.Goal{
			Objective: goal,
		},
		CurrentDiffBP: 10000,
	}
}

func TestEngine_Assess_BasisPoint_Transitions(t *testing.T) {
	ctx := context.Background()
	mockClock := NewMockClock(time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC))

	currentScore := 100.0
	evalFunc := dck.EvaluatorFunc(func(ctx context.Context, start, current, goal string) (float64, error) {
		return currentScore, nil
	})

	engine := dck.NewEngine(
		evalFunc,
		dck.WithClock(mockClock),
		dck.WithStagnantWindow(3),
	)

	c := setupTestCapsule("capsule-bp-1", "start", "goal")

	// 1回目 (80.0% = 8000 BP) -> IN_PROGRESS
	currentScore = 80.0
	res1, err := engine.Assess(ctx, c, "state-1")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if res1.DiffBP != 8000 {
		t.Errorf("expected 8000 BP, got %d", res1.DiffBP)
	}
	if res1.Status != dck.StatusInProgress {
		t.Errorf("expected STATUS_IN_PROGRESS, got %s", res1.Status)
	}

	// 2回目 (60.0% = 6000 BP) -> IMPROVING
	mockClock.Advance(1 * time.Second)
	currentScore = 60.0
	res2, _ := engine.Assess(ctx, c, "state-2")
	if res2.Status != dck.StatusImproving {
		t.Errorf("expected STATUS_IMPROVING, got %s", res2.Status)
	}

	// 3回目 (70.0% = 7000 BP) -> REGRESSING
	mockClock.Advance(1 * time.Second)
	currentScore = 70.0
	res3, _ := engine.Assess(ctx, c, "state-3")
	if res3.Status != dck.StatusRegressing {
		t.Errorf("expected STATUS_REGRESSING, got %s", res3.Status)
	}

	// 4回目〜5回目 (変化なし -> STAGNANT)
	mockClock.Advance(1 * time.Second)
	_, _ = engine.Assess(ctx, c, "state-4")

	mockClock.Advance(1 * time.Second)
	res5, _ := engine.Assess(ctx, c, "state-5")
	if res5.Status != dck.StatusStagnant {
		t.Errorf("expected STATUS_STAGNANT, got %s", res5.Status)
	}

	// 0.0% = 0 BP -> CONVERGED
	currentScore = 0.0
	res6, _ := engine.Assess(ctx, c, "goal")
	if res6.Status != dck.StatusConverged {
		t.Errorf("expected STATUS_CONVERGED, got %s", res6.Status)
	}
}

func TestEngine_PassGate_FailGate(t *testing.T) {
	mockClock := NewMockClock(time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC))
	engine := dck.NewEngine(dck.NewRuleBasedEvaluator(), dck.WithClock(mockClock))
	c := setupTestCapsule("capsule-gate-1", "start", "goal")

	engine.PassGate(c, "gate-1", "passed gate 1")
	engine.FailGate(c, "gate-2", "failed gate 2")

	if len(c.Gates) != 2 {
		t.Fatalf("expected 2 gates, got %d", len(c.Gates))
	}
}

func TestEngine_Concurrent_Race(t *testing.T) {
	ctx := context.Background()
	engine := dck.NewEngine(dck.NewRuleBasedEvaluator())
	c := setupTestCapsule("concurrent-capsule", "start", "target")

	var wg sync.WaitGroup
	workers := 10
	iterations := 50

	for i := 0; i < workers; i++ {
		wg.Add(1)
		go func(workerID int) {
			defer wg.Done()
			for j := 0; j < iterations; j++ {
				state := fmt.Sprintf("state-%d-%d", workerID, j)
				_, err := engine.Assess(ctx, c, state)
				if err != nil {
					t.Errorf("assess failed: %v", err)
				}
				_ = engine.GetMetrics()
			}
		}(i)
	}

	wg.Wait()

	metrics := engine.GetMetrics()
	expectedTotal := uint64(workers * iterations)
	if metrics.TotalAssessments != expectedTotal {
		t.Errorf("expected total assessments %d, got %d", expectedTotal, metrics.TotalAssessments)
	}
}

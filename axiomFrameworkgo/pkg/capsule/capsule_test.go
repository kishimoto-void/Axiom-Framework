package capsule_test

import (
	"bytes"
	"crypto/sha256"
	"encoding/json"
	"math"
	"os"
	"path/filepath"
	"sync"
	"testing"
	"time"

	"miniaxiom/pkg/capsule"
)

// fixedTimes provides a fully deterministic clock sequence for golden tests.
var fixedTimes = []time.Time{
	time.Date(2026, 8, 6, 12, 0, 0, 0, time.UTC), // t0: CreatedAt / UpdatedAt
	time.Date(2026, 8, 6, 12, 0, 1, 0, time.UTC), // t1: first UpdateDifference
	time.Date(2026, 8, 6, 12, 0, 2, 0, time.UTC), // t2: first Gate
	time.Date(2026, 8, 6, 12, 0, 3, 0, time.UTC), // t3: second Gate
	time.Date(2026, 8, 6, 12, 0, 4, 0, time.UTC), // t4: second UpdateDifference
}

func newGoldenCapsule(t *testing.T) *capsule.Capsule {
	t.Helper()
	clock, err := capsule.NewMockClock(fixedTimes...)
	if err != nil {
		t.Fatalf("NewMockClock: %v", err)
	}

	c, err := capsule.NewDeterministic("System Architect", "initial", "reach convergence", clock)
	if err != nil {
		t.Fatalf("NewDeterministic: %v", err)
	}

	// Add ordered constraints (map will be sorted on JSON marshal)
	c.Constraints["priority"] = "high"
	c.Constraints["max_latency_ms"] = "100"

	// Context
	c.Context = append(c.Context, "context-item-1", "context-item-2")

	// Update difference (consumes t1)
	if err := c.UpdateDifference(capsule.MustBasisPoint(7500), clock.Now()); err != nil {
		t.Fatalf("UpdateDifference: %v", err)
	}

	// Gates (consume t2, t3)
	if err := c.AddGate("gate-alpha", true, "first checkpoint", clock); err != nil {
		t.Fatalf("AddGate alpha: %v", err)
	}
	if err := c.AddGate("gate-beta", false, "second checkpoint", clock); err != nil {
		t.Fatalf("AddGate beta: %v", err)
	}

	// Final update (consumes t4)
	if err := c.UpdateDifference(capsule.MustBasisPoint(3200), clock.Now()); err != nil {
		t.Fatalf("UpdateDifference final: %v", err)
	}

	if err := c.AssertInvariants(); err != nil {
		t.Fatalf("AssertInvariants: %v", err)
	}
	return c
}

func TestGoldenJSON(t *testing.T) {
	c := newGoldenCapsule(t)

	data, err := json.MarshalIndent(c, "", "  ")
	if err != nil {
		t.Fatalf("MarshalIndent: %v", err)
	}

	goldenPath := filepath.Join("testdata", "capsule_v1.golden.json")

	if os.Getenv("UPDATE_GOLDEN") == "true" {
		_ = os.MkdirAll("testdata", 0755)
		if err := os.WriteFile(goldenPath, data, 0644); err != nil {
			t.Fatalf("write golden: %v", err)
		}
		t.Logf("updated golden file: %s", goldenPath)
	}

	expected, err := os.ReadFile(goldenPath)
	if err != nil {
		t.Skipf("golden file %s not found; run with UPDATE_GOLDEN=true", goldenPath)
	}

	// Normalize both sides (trim trailing whitespace differences)
	got := bytes.TrimSpace(data)
	want := bytes.TrimSpace(expected)

	if !bytes.Equal(got, want) {
		t.Errorf("Golden JSON mismatch!\n--- got ---\n%s\n--- want ---\n%s", got, want)
	}

	// Also verify hash stability
	h := sha256.Sum256(got)
	t.Logf("SHA256(capsule_v1.golden.json) = %x", h)
}

func TestGoldenHashStability(t *testing.T) {
	const iterations = 1000
	var firstHash [32]byte

	for i := 0; i < iterations; i++ {
		c := newGoldenCapsule(t)
		data, err := json.Marshal(c) // compact for pure content hash
		if err != nil {
			t.Fatalf("Marshal: %v", err)
		}
		h := sha256.Sum256(data)
		if i == 0 {
			firstHash = h
			continue
		}
		if h != firstHash {
			t.Fatalf("non-deterministic JSON at iteration %d\nfirst: %x\ngot:   %x", i, firstHash, h)
		}
	}
	t.Logf("Determinism PASS (%d runs) SHA256=%x", iterations, firstHash)
}

func TestInvariants_ZeroEvaluatedAtRejected(t *testing.T) {
	clock, _ := capsule.NewMockClock(fixedTimes[0])
	c, err := capsule.NewDeterministic("p", "s", "g", clock)
	if err != nil {
		t.Fatal(err)
	}

	// Manually inject a zero-time gate (bypassing AddGate)
	c.Gates = append(c.Gates, capsule.SectionGate{
		Name:        "bad-gate",
		Passed:      true,
		Description: "should fail invariant",
		EvaluatedAt: time.Time{}, // zero
	})

	if err := c.AssertInvariants(); err == nil {
		t.Fatal("expected invariant violation for zero EvaluatedAt, got nil")
	}
}

func TestInvariants_ZeroKnowledgeContextConflict(t *testing.T) {
	clock, _ := capsule.NewMockClock(fixedTimes[0])
	c, err := capsule.NewDeterministic("p", "s", "g", clock)
	if err != nil {
		t.Fatal(err)
	}

	c.ZeroKnowledge = true
	c.Context = []string{"secret"}

	if err := c.AssertInvariants(); err == nil {
		t.Fatal("expected ZeroKnowledge + non-empty Context to violate invariants")
	}
}

func TestUpdateDifference_Bounds(t *testing.T) {
	clock, _ := capsule.NewMockClock(fixedTimes...)
	c, err := capsule.NewDeterministic("p", "s", "g", clock)
	if err != nil {
		t.Fatal(err)
	}

	// Out of range must fail
	if err := c.UpdateDifference(capsule.BasisPoint(-1), clock.Now()); err == nil {
		t.Error("expected error for negative BP")
	}
	if err := c.UpdateDifference(capsule.BasisPoint(10001), clock.Now()); err == nil {
		t.Error("expected error for BP > 10000")
	}

	// Valid update
	if err := c.UpdateDifference(capsule.MustBasisPoint(0), clock.Now()); err != nil {
		t.Errorf("valid 0 BP update failed: %v", err)
	}
	if !c.IsConverged() {
		t.Error("expected IsConverged() == true after DiffBP=0")
	}
}

func TestAddGate_RequiresClock(t *testing.T) {
	clock, _ := capsule.NewMockClock(fixedTimes[0])
	c, err := capsule.NewDeterministic("p", "s", "g", clock)
	if err != nil {
		t.Fatal(err)
	}

	if err := c.AddGate("g", true, "d", nil); err == nil {
		t.Error("expected error when clock is nil")
	}
}

func TestIsConverged(t *testing.T) {
	clock, _ := capsule.NewMockClock(fixedTimes...)
	c, err := capsule.NewDeterministic("p", "s", "g", clock)
	if err != nil {
		t.Fatal(err)
	}

	// Default TolBP = 500, CurrentDiffBP = 10000 → not converged
	if c.IsConverged() {
		t.Error("expected not converged at creation")
	}

	_ = c.UpdateDifference(capsule.MustBasisPoint(400), clock.Now()) // 4.00% < 5.00%
	if !c.IsConverged() {
		t.Error("expected converged when DiffBP <= TolBP")
	}
}

func TestBasisPoint_Edges(t *testing.T) {
	if _, err := capsule.NewBasisPoint(-1); err == nil {
		t.Error("expected error for -1")
	}
	if _, err := capsule.NewBasisPoint(10001); err == nil {
		t.Error("expected error for 10001")
	}
	bp, err := capsule.NewBasisPoint(0)
	if err != nil || bp != 0 {
		t.Errorf("NewBasisPoint(0) failed: %v %d", err, bp)
	}
	bp, err = capsule.NewBasisPoint(10000)
	if err != nil || bp != 10000 {
		t.Errorf("NewBasisPoint(10000) failed: %v %d", err, bp)
	}

	if _, err := capsule.FloatToBP(math.NaN()); err == nil {
		t.Error("FloatToBP(NaN) should fail")
	}
	if _, err := capsule.FloatToBP(math.Inf(1)); err == nil {
		t.Error("FloatToBP(+Inf) should fail")
	}

	bp, err = capsule.FloatToBP(42.567)
	if err != nil {
		t.Fatal(err)
	}
	// 42.567 * 100 = 4256.7 → Round → 4257
	if bp != 4257 {
		t.Errorf("FloatToBP(42.567) = %d, want 4257", bp)
	}
}

func TestBasisPoint_FloatRoundTrip(t *testing.T) {
	bp, err := capsule.FloatToBP(0.0)
	if err != nil || bp != 0 {
		t.Errorf("FloatToBP(0) = %d, %v", bp, err)
	}
	bp, err = capsule.FloatToBP(100.0)
	if err != nil || bp != 10000 {
		t.Errorf("FloatToBP(100) = %d, %v", bp, err)
	}
	if bp.Float64() != 100.0 {
		t.Errorf("Float64() = %f", bp.Float64())
	}
}

func TestConcurrentAccess(t *testing.T) {
	clock, _ := capsule.NewMockClock(fixedTimes...)
	c, err := capsule.NewDeterministic("p", "s", "g", clock)
	if err != nil {
		t.Fatal(err)
	}

	var wg sync.WaitGroup
	const workers = 20
	const iters = 100

	for i := 0; i < workers; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			for j := 0; j < iters; j++ {
				_ = c.GetDifference()
				_ = c.GetTolerance()
				_ = c.IsConverged()
				_ = c.AssertInvariants()
				// occasional update
				if j%10 == 0 {
					bp := capsule.MustBasisPoint(int64(1000 + (id*iters+j)%8000))
					_ = c.UpdateDifference(bp, time.Now().UTC())
				}
			}
		}(i)
	}
	wg.Wait()
}

func TestSaveLoadRoundTrip(t *testing.T) {
	c := newGoldenCapsule(t)

	dir := t.TempDir()
	path := filepath.Join(dir, "capsule.json")

	if err := c.Save(path); err != nil {
		t.Fatalf("Save: %v", err)
	}

	loaded, err := capsule.Load(path)
	if err != nil {
		t.Fatalf("Load: %v", err)
	}

	// Compare key fields
	if loaded.Persona != c.Persona {
		t.Errorf("Persona mismatch: %q vs %q", loaded.Persona, c.Persona)
	}
	if loaded.CurrentDiffBP != c.CurrentDiffBP {
		t.Errorf("CurrentDiffBP mismatch: %d vs %d", loaded.CurrentDiffBP, c.CurrentDiffBP)
	}
	if len(loaded.Gates) != len(c.Gates) {
		t.Errorf("Gates length mismatch: %d vs %d", len(loaded.Gates), len(c.Gates))
	}
	for i := range c.Gates {
		if loaded.Gates[i].Name != c.Gates[i].Name ||
			loaded.Gates[i].Passed != c.Gates[i].Passed ||
			!loaded.Gates[i].EvaluatedAt.Equal(c.Gates[i].EvaluatedAt) {
			t.Errorf("Gate[%d] mismatch: %+v vs %+v", i, loaded.Gates[i], c.Gates[i])
		}
	}

	if err := loaded.AssertInvariants(); err != nil {
		t.Errorf("loaded capsule failed invariants: %v", err)
	}
}

func TestMockClock_Exhaustion(t *testing.T) {
	ts := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
	clock, err := capsule.NewMockClock(ts)
	if err != nil {
		t.Fatal(err)
	}
	if got := clock.Now(); !got.Equal(ts) {
		t.Errorf("first Now() = %v", got)
	}
	// subsequent calls should return the last value
	if got := clock.Now(); !got.Equal(ts) {
		t.Errorf("exhausted Now() = %v, want last value", got)
	}
}

func TestNewMockClock_EmptyRejected(t *testing.T) {
	if _, err := capsule.NewMockClock(); err == nil {
		t.Error("expected error for empty times")
	}
}

func BenchmarkNewDeterministic(b *testing.B) {
	clock, _ := capsule.NewMockClock(fixedTimes...)
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		_, _ = capsule.NewDeterministic("p", "s", "g", clock)
	}
}

func BenchmarkUpdateDifference(b *testing.B) {
	clock, _ := capsule.NewMockClock(fixedTimes...)
	c, _ := capsule.NewDeterministic("p", "s", "g", clock)
	bp := capsule.MustBasisPoint(5000)
	ts := time.Now().UTC()
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = c.UpdateDifference(bp, ts)
	}
}

func BenchmarkAssertInvariants(b *testing.B) {
	c := newGoldenCapsule(&testing.T{})
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = c.AssertInvariants()
	}
}

func BenchmarkJSONMarshal(b *testing.B) {
	c := newGoldenCapsule(&testing.T{})
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = json.Marshal(c)
	}
}

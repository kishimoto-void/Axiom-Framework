package capsule

import (
	"fmt"
	"time"
)

// Clock provides an interface for time generation to enable deterministic execution.
type Clock interface {
	Now() time.Time
}

// SystemClock uses real system time.
type SystemClock struct{}

func (SystemClock) Now() time.Time {
	return time.Now().UTC()
}

// MockClock provides sequence-based deterministic timestamps for testing.
type MockClock struct {
	Times []time.Time
	Index int
}

func NewMockClock(times ...time.Time) (*MockClock, error) {
	if len(times) == 0 {
		return nil, fmt.Errorf("mock clock requires at least one timestamp")
	}
	return &MockClock{Times: times, Index: 0}, nil
}

func (m *MockClock) Now() time.Time {
	if len(m.Times) == 0 {
		return time.Time{}
	}
	if m.Index >= len(m.Times) {
		return m.Times[len(m.Times)-1]
	}
	t := m.Times[m.Index]
	m.Index++
	return t
}

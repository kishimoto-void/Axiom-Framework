package dck

import (
	"fmt"
	"sync"
	"time"

	"miniaxiom/pkg/capsule"
)

type State string

const (
	StateProgressing State = "PROGRESSING"
	StateConverged   State = "CONVERGED"
	StateStagnant    State = "STAGNANT"
)

type EngineConfig struct {
	MinDeltaBP        capsule.BasisPoint `json:"min_delta_bp"`
	MaxStagnantCount  int                `json:"max_stagnant_count"`
	SmoothingFactorBP int64              `json:"smoothing_factor_bp"` // e.g. 7000 = 70.00%
	MaxHistory        int                `json:"max_history"`
}

func (cfg EngineConfig) Validate() error {
	if cfg.MinDeltaBP < capsule.BPMin || cfg.MinDeltaBP > capsule.BPMax {
		return fmt.Errorf("invalid MinDeltaBP: %d", cfg.MinDeltaBP)
	}
	if cfg.MaxStagnantCount <= 0 {
		return fmt.Errorf("MaxStagnantCount must be greater than 0")
	}
	if cfg.SmoothingFactorBP <= 0 || cfg.SmoothingFactorBP > 10000 {
		return fmt.Errorf("SmoothingFactorBP must be in range (0, 10000]: %d", cfg.SmoothingFactorBP)
	}
	if cfg.MaxHistory <= 0 {
		return fmt.Errorf("MaxHistory must be greater than 0")
	}
	return nil
}

func DefaultConfig() EngineConfig {
	return EngineConfig{
		MinDeltaBP:        capsule.MustBasisPoint(50), // 0.50%
		MaxStagnantCount:  3,
		SmoothingFactorBP: 7000, // 70.00%
		MaxHistory:        1000,
	}
}

type EvaluationResult struct {
	Timestamp    time.Time          `json:"timestamp"`
	PreviousDiff capsule.BasisPoint `json:"previous_diff"`
	NewDiff      capsule.BasisPoint `json:"new_diff"`
	Delta        capsule.BasisPoint `json:"delta"`
	State        State              `json:"state"`
}

type Engine struct {
	mu              sync.RWMutex
	clock           capsule.Clock
	cfg             EngineConfig
	stagnantCounter int
	historyRing     []EvaluationResult // 固定長バッファ
	head            int                // 最古の要素のインデックス
	count           int                // 現在格納されている要素数
}

func NewDeterministicEngine(clock capsule.Clock, cfg EngineConfig) (*Engine, error) {
	if clock == nil {
		return nil, fmt.Errorf("clock dependency must not be nil")
	}
	if err := cfg.Validate(); err != nil {
		return nil, fmt.Errorf("invalid engine config: %w", err)
	}

	return &Engine{
		clock:       clock,
		cfg:         cfg,
		historyRing: make([]EvaluationResult, cfg.MaxHistory),
		head:        0,
		count:       0,
	}, nil
}

// divRoundHalfUp performs deterministic half-up integer rounding division for non-negative values.
func divRoundHalfUp(numerator, denominator int64) int64 {
	if denominator == 0 {
		panic("division by zero in divRoundHalfUp")
	}
	return (numerator + denominator/2) / denominator
}

// Evaluate evaluates difference reduction deterministically without side effects on Capsule.
// O(1) time complexity ring-buffer update.
func (e *Engine) Evaluate(prevDiff, tolerance, rawTargetBP capsule.BasisPoint) (EvaluationResult, error) {
	if rawTargetBP < capsule.BPMin || rawTargetBP > capsule.BPMax {
		return EvaluationResult{}, fmt.Errorf("rawTargetBP %d out of bounds", rawTargetBP)
	}

	e.mu.Lock()
	defer e.mu.Unlock()

	numerator := (e.cfg.SmoothingFactorBP * int64(rawTargetBP)) + ((10000 - e.cfg.SmoothingFactorBP) * int64(prevDiff))
	smoothedRaw := divRoundHalfUp(numerator, 10000)
	smoothedBP, err := capsule.NewBasisPoint(smoothedRaw)
	if err != nil {
		return EvaluationResult{}, fmt.Errorf("calculated smoothed BP invalid: %w", err)
	}

	deltaBP := prevDiff - smoothedBP
	now := e.clock.Now()

	var nextState State
	if smoothedBP <= tolerance {
		e.stagnantCounter = 0
		nextState = StateConverged
	} else {
		if deltaBP < e.cfg.MinDeltaBP {
			e.stagnantCounter++
		} else {
			e.stagnantCounter = 0
		}

		if e.stagnantCounter >= e.cfg.MaxStagnantCount {
			nextState = StateStagnant
		} else {
			nextState = StateProgressing
		}
	}

	result := EvaluationResult{
		Timestamp:    now,
		PreviousDiff: prevDiff,
		NewDiff:      smoothedBP,
		Delta:        deltaBP,
		State:        nextState,
	}

	// 固定長リングバッファへの O(1) 挿入
	if e.count < e.cfg.MaxHistory {
		tail := (e.head + e.count) % e.cfg.MaxHistory
		e.historyRing[tail] = result
		e.count++
	} else {
		e.historyRing[e.head] = result
		e.head = (e.head + 1) % e.cfg.MaxHistory
	}

	return result, nil
}

// GetHistory returns evaluation history ordered chronologically (oldest to newest).
func (e *Engine) GetHistory() []EvaluationResult {
	e.mu.RLock()
	defer e.mu.RUnlock()

	result := make([]EvaluationResult, e.count)
	for i := 0; i < e.count; i++ {
		idx := (e.head + i) % e.cfg.MaxHistory
		result[i] = e.historyRing[idx]
	}
	return result
}

func (e *Engine) ResetStagnation() {
	e.mu.Lock()
	defer e.mu.Unlock()
	e.stagnantCounter = 0
}

package capsule

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"sync"
	"time"
)

const DefaultVersion = "0.2.0"

type Goal struct {
	Objective string     `json:"objective"`
	TargetBP  BasisPoint `json:"target_bp"`
	TolBP     BasisPoint `json:"tolerance_bp"`
}

type SectionGate struct {
	Name        string    `json:"name"`
	Passed      bool      `json:"passed"`
	Description string    `json:"description"`
	EvaluatedAt time.Time `json:"evaluated_at"`
}

type Capsule struct {
	mu            sync.RWMutex
	Version       string            `json:"version"`
	Persona       string            `json:"persona"`
	StartState    string            `json:"start_state"`
	Goal          Goal              `json:"goal"`
	CurrentDiffBP BasisPoint        `json:"current_difference_bp"`
	Constraints   map[string]string `json:"constraints"`
	Context       []string          `json:"context"`
	Gates         []SectionGate     `json:"gates"`
	ZeroKnowledge bool              `json:"zero_knowledge"`
	CreatedAt     time.Time         `json:"created_at"`
	UpdatedAt     time.Time         `json:"updated_at"`
}

func NewDeterministic(persona, startState, objective string, clock Clock) (*Capsule, error) {
	if clock == nil {
		return nil, fmt.Errorf("clock dependency must not be nil")
	}

	now := clock.Now()
	c := &Capsule{
		Version:    DefaultVersion,
		Persona:    persona,
		StartState: startState,
		Goal: Goal{
			Objective: objective,
			TargetBP:  BPMax,
			TolBP:     MustBasisPoint(500), // 5.00%
		},
		CurrentDiffBP: BPMax,
		Constraints:   make(map[string]string),
		Context:       make([]string, 0),
		Gates:         make([]SectionGate, 0),
		ZeroKnowledge: false,
		CreatedAt:     now,
		UpdatedAt:     now,
	}

	c.Normalize(now)
	if err := c.AssertInvariants(); err != nil {
		return nil, fmt.Errorf("failed invariant assertion on creation: %w", err)
	}
	return c, nil
}

func (c *Capsule) AssertInvariants() error {
	c.mu.RLock()
	defer c.mu.RUnlock()

	if c.CurrentDiffBP < BPMin || c.CurrentDiffBP > BPMax {
		return fmt.Errorf("invariant violated: CurrentDiffBP %d out of bounds", c.CurrentDiffBP)
	}
	if c.Goal.TolBP < BPMin || c.Goal.TolBP > BPMax {
		return fmt.Errorf("invariant violated: Goal.TolBP %d out of bounds", c.Goal.TolBP)
	}
	if c.Goal.TargetBP < BPMin || c.Goal.TargetBP > BPMax {
		return fmt.Errorf("invariant violated: Goal.TargetBP %d out of bounds", c.Goal.TargetBP)
	}
	if c.CreatedAt.IsZero() {
		return fmt.Errorf("invariant violated: CreatedAt must not be zero time")
	}
	if c.UpdatedAt.Before(c.CreatedAt) {
		return fmt.Errorf("invariant violated: UpdatedAt (%v) is before CreatedAt (%v)", c.UpdatedAt, c.CreatedAt)
	}
	if c.ZeroKnowledge && len(c.Context) > 0 {
		return fmt.Errorf("invariant violated: ZeroKnowledge is true but context has %d items", len(c.Context))
	}
	for i, gate := range c.Gates {
		if gate.EvaluatedAt.IsZero() {
			return fmt.Errorf("invariant violated: Gate[%d] '%s' has zero EvaluatedAt timestamp", i, gate.Name)
		}
	}
	return nil
}

func (c *Capsule) Normalize(now time.Time) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.normalizeUnlocked(now)
}

func (c *Capsule) normalizeUnlocked(now time.Time) {
	c.Persona = strings.TrimSpace(c.Persona)
	c.StartState = strings.TrimSpace(c.StartState)
	c.Goal.Objective = strings.TrimSpace(c.Goal.Objective)
	if c.Version == "" {
		c.Version = DefaultVersion
	}
	if c.Constraints == nil {
		c.Constraints = make(map[string]string)
	}
	if c.Context == nil {
		c.Context = make([]string, 0)
	}
	if c.Gates == nil {
		c.Gates = make([]SectionGate, 0)
	}
	if c.ZeroKnowledge && len(c.Context) > 0 {
		c.Context = make([]string, 0)
	}
	if c.CreatedAt.IsZero() {
		c.CreatedAt = now
	}
	if c.UpdatedAt.IsZero() || c.UpdatedAt.Before(c.CreatedAt) {
		c.UpdatedAt = c.CreatedAt
	}
}

func Load(path string) (*Capsule, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read capsule file: %w", err)
	}
	var c Capsule
	if err := json.Unmarshal(data, &c); err != nil {
		return nil, fmt.Errorf("failed to unmarshal capsule JSON: %w", err)
	}
	if err := c.AssertInvariants(); err != nil {
		return nil, fmt.Errorf("loaded capsule failed invariant check: %w", err)
	}
	return &c, nil
}

func (c *Capsule) Save(path string) error {
	c.mu.Lock()
	if err := c.assertInvariantsUnlocked(); err != nil {
		c.mu.Unlock()
		return fmt.Errorf("cannot save capsule with invalid invariants: %w", err)
	}
	data, err := json.MarshalIndent(c, "", "  ")
	c.mu.Unlock()
	if err != nil {
		return fmt.Errorf("failed to marshal capsule: %w", err)
	}
	return os.WriteFile(path, data, 0644)
}

func (c *Capsule) IsConverged() bool {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.CurrentDiffBP <= c.Goal.TolBP
}

func (c *Capsule) UpdateDifference(diffBP BasisPoint, timestamp time.Time) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if diffBP < BPMin || diffBP > BPMax {
		return fmt.Errorf("cannot update difference: value %d out of bounds", diffBP)
	}

	c.CurrentDiffBP = diffBP
	if timestamp.After(c.UpdatedAt) {
		c.UpdatedAt = timestamp
	}
	return c.assertInvariantsUnlocked()
}

func (c *Capsule) AddGate(name string, passed bool, description string, clock Clock) error {
	if clock == nil {
		return fmt.Errorf("clock dependency must not be nil")
	}

	c.mu.Lock()
	defer c.mu.Unlock()

	now := clock.Now()
	gate := SectionGate{
		Name:        strings.TrimSpace(name),
		Passed:      passed,
		Description: strings.TrimSpace(description),
		EvaluatedAt: now,
	}

	c.Gates = append(c.Gates, gate)
	if now.After(c.UpdatedAt) {
		c.UpdatedAt = now
	}
	return c.assertInvariantsUnlocked()
}

func (c *Capsule) GetDifference() BasisPoint {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.CurrentDiffBP
}

func (c *Capsule) GetTolerance() BasisPoint {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.Goal.TolBP
}

func (c *Capsule) assertInvariantsUnlocked() error {
	if c.CurrentDiffBP < BPMin || c.CurrentDiffBP > BPMax {
		return fmt.Errorf("invariant violated: CurrentDiffBP %d out of bounds", c.CurrentDiffBP)
	}
	if c.ZeroKnowledge && len(c.Context) > 0 {
		return fmt.Errorf("invariant violated: ZeroKnowledge is true but context is non-empty")
	}
	for i, gate := range c.Gates {
		if gate.EvaluatedAt.IsZero() {
			return fmt.Errorf("invariant violated: Gate[%d] '%s' has zero EvaluatedAt timestamp", i, gate.Name)
		}
	}
	return nil
}

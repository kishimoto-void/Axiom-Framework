// Package dck は、ACPカプセルの動的な収束ライフサイクルおよび評価機能を提供します。
package dck

import (
	"context"
	"errors"
	"fmt"
	"math"
	"strings"
	"sync"

	"miniaxiom/pkg/capsule"
)

var (
	// ErrNilEvaluator は評価器に nil が指定された場合のエラーです。
	ErrNilEvaluator = errors.New("evaluator cannot be nil")
	// ErrNoEvaluator は評価器が設定されていない場合のエラーです。
	ErrNoEvaluator = errors.New("evaluator is not configured")
	// ErrNoEvaluators は WeightedEvaluator に評価器が1つも登録されていない場合のエラーです。
	ErrNoEvaluators = errors.New("no evaluators configured in WeightedEvaluator")
	// ErrInvalidWeight は重み付けの値が不正（0以下）な場合のエラーです。
	ErrInvalidWeight = errors.New("weight must be greater than zero")
	// ErrZeroTotalWeight は登録された重みの合計が0の場合のエラーです。
	ErrZeroTotalWeight = errors.New("total weight cannot be zero")
)

// Evaluator は現在の状態と目標状態の差異を評価するインターフェースです。
// 戻り値は float64 スコア（0.0 〜 100.0）とし、内部で BasisPoint へ正規化されます。
type Evaluator interface {
	Evaluate(ctx context.Context, start, current, goal string) (float64, error)
}

// EvaluatorFunc は標準的な関数を Evaluator インターフェースとして扱うためのアダプターです。
type EvaluatorFunc func(ctx context.Context, start, current, goal string) (float64, error)

// Evaluate は関数自体を実行して評価スコアを返します。
func (f EvaluatorFunc) Evaluate(ctx context.Context, start, current, goal string) (float64, error) {
	return f(ctx, start, current, goal)
}

// ManualEvaluator は指定された固定のスコアを常に返す評価器です。
type ManualEvaluator struct {
	score float64
}

// NewManualEvaluator は指定されたスコアを持つ ManualEvaluator のインスタンスを生成します。
func NewManualEvaluator(score float64) *ManualEvaluator {
	return &ManualEvaluator{score: clampScore(score)}
}

// Evaluate は設定された固定スコアを返します。
func (e *ManualEvaluator) Evaluate(_ context.Context, _, _, _ string) (float64, error) {
	return e.score, nil
}

// RuleBasedEvaluator は文字列比較を行うサンプル・デモ用の評価器です。
type RuleBasedEvaluator struct{}

// NewRuleBasedEvaluator は RuleBasedEvaluator のインスタンスを生成します。
func NewRuleBasedEvaluator() *RuleBasedEvaluator {
	return &RuleBasedEvaluator{}
}

// Evaluate は文字列の一致度に基づき簡易スコアを計算します。
func (e *RuleBasedEvaluator) Evaluate(_ context.Context, _, current, goal string) (float64, error) {
	current = strings.TrimSpace(current)
	goal = strings.TrimSpace(goal)
	if current == "" {
		return 100.0, nil
	}
	if current == goal {
		return 0.0, nil
	}
	if strings.Contains(current, goal) || strings.Contains(goal, current) {
		return 25.0, nil
	}
	return 50.0, nil
}

type weightedItem struct {
	evaluator Evaluator
	weight    float64
}

// WeightedEvaluator は複数の評価器に重み付けをして総合評価を行う複合評価器です。
type WeightedEvaluator struct {
	mu    sync.RWMutex
	items []weightedItem
}

// NewWeightedEvaluator は空の WeightedEvaluator のインスタンスを生成します。
func NewWeightedEvaluator() *WeightedEvaluator {
	return &WeightedEvaluator{
		items: make([]weightedItem, 0),
	}
}

// Add は評価器と重みを設定して追加します。
func (e *WeightedEvaluator) Add(eval Evaluator, weight float64) error {
	if eval == nil {
		return ErrNilEvaluator
	}
	if weight <= 0 {
		return ErrInvalidWeight
	}
	e.mu.Lock()
	defer e.mu.Unlock()
	e.items = append(e.items, weightedItem{
		evaluator: eval,
		weight:    weight,
	})
	return nil
}

// AddFunc は関数型の評価器と重みを設定して追加します。
func (e *WeightedEvaluator) AddFunc(fn EvaluatorFunc, weight float64) error {
	return e.Add(fn, weight)
}

// Evaluate は登録された全ての評価器を実行し、重み付き平均スコアを算出します。
func (e *WeightedEvaluator) Evaluate(ctx context.Context, start, current, goal string) (float64, error) {
	e.mu.RLock()
	if len(e.items) == 0 {
		e.mu.RUnlock()
		return 100.0, ErrNoEvaluators
	}
	itemsCopy := make([]weightedItem, len(e.items))
	copy(itemsCopy, e.items)
	e.mu.RUnlock()

	var totalWeightedScore float64
	var totalWeight float64

	for i, item := range itemsCopy {
		select {
		case <-ctx.Done():
			return 0.0, ctx.Err()
		default:
		}
		score, err := item.evaluator.Evaluate(ctx, start, current, goal)
		if err != nil {
			return 0.0, fmt.Errorf("evaluator [%d] failed: %w", i, err)
		}
		totalWeightedScore += clampScore(score) * item.weight
		totalWeight += item.weight
	}

	if totalWeight <= 0 {
		return 100.0, ErrZeroTotalWeight
	}
	return clampScore(totalWeightedScore / totalWeight), nil
}

func clampScore(score float64) float64 {
	if math.IsNaN(score) || score < 0.0 {
		return 0.0
	}
	if score > 100.0 {
		return 100.0
	}
	return score
}

// ScoreToBasisPoint は float64 のスコア（0.0 〜 100.0）を BasisPoint 型（0 〜 10000 BP）に変換します。
func ScoreToBasisPoint(score float64) capsule.BasisPoint {
	clamped := clampScore(score)
	return capsule.BasisPoint(math.Round(clamped * 100.0))
}

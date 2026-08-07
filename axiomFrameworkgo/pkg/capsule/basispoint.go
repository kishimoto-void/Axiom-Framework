package capsule

import (
	"fmt"
	"math"
)

// BasisPoint represents percentage * 100 in integer domain (0 to 10000 = 0.00% to 100.00%).
type BasisPoint int64

const (
	BPMin BasisPoint = 0
	BPMax BasisPoint = 10000
)

func NewBasisPoint(v int64) (BasisPoint, error) {
	if v < int64(BPMin) || v > int64(BPMax) {
		return 0, fmt.Errorf("basis point out of valid range [0, 10000]: %d", v)
	}
	return BasisPoint(v), nil
}

func MustBasisPoint(v int64) BasisPoint {
	bp, err := NewBasisPoint(v)
	if err != nil {
		panic(err)
	}
	return bp
}

func FloatToBP(val float64) (BasisPoint, error) {
	if math.IsNaN(val) || math.IsInf(val, 0) {
		return 0, fmt.Errorf("invalid float value: NaN or Inf")
	}
	rounded := math.Round(val * 100.0)
	v := int64(rounded)
	return NewBasisPoint(v)
}

func (bp BasisPoint) Float64() float64 {
	return float64(bp) / 100.0
}

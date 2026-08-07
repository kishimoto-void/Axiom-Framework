package pss

import (
	"bufio"
	"bytes"
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"time"

	"miniaxiom/pkg/capsule"
)

// appendBP appends the string representation of BasisPoint to buf without allocation, overflow-safe.
func appendBP(buf []byte, bp capsule.BasisPoint) []byte {
	v := int64(bp)
	var u uint64
	if v < 0 {
		buf = append(buf, '-')
		u = uint64(-(v + 1)) + 1
	} else {
		u = uint64(v)
	}
	whole := u / 100
	frac := u % 100
	buf = strconv.AppendUint(buf, whole, 10)
	buf = append(buf, '.')
	if frac < 10 {
		buf = append(buf, '0')
	}
	return strconv.AppendUint(buf, frac, 10)
}

func needsQuote(s string) bool {
	if len(s) == 0 {
		return false
	}
	if s[0] == ' ' || s[len(s)-1] == ' ' {
		return true
	}
	for i := 0; i < len(s); i++ {
		b := s[i]
		if b == ':' || b == '#' || b == '|' || b == '\n' || b == '\r' || b == '"' {
			return true
		}
	}
	return false
}

func formatValue(s string) string {
	if needsQuote(s) {
		return strconv.Quote(s)
	}
	return s
}

// FormatWriter writes a Capsule instance as PSS syntax directly to an io.Writer stream.
func FormatWriter(ctx context.Context, w io.Writer, c *capsule.Capsule) error {
	if ctx == nil {
		return ErrNilContext
	}
	if c == nil {
		return ErrNilCapsule
	}
	if w == nil {
		return ErrNilWriter
	}

	if err := ctx.Err(); err != nil {
		return err
	}

	if err := c.AssertInvariants(); err != nil {
		return fmt.Errorf("pss: cannot format capsule violating invariants: %w", err)
	}

	bw := bufio.NewWriter(w)
	var scratch []byte

	writeBytes := func(b []byte) error {
		_, err := bw.Write(b)
		if err == nil {
			err = bw.WriteByte('\n')
		}
		return err
	}

	writeString := func(s string) error {
		_, err := bw.WriteString(s)
		if err == nil {
			err = bw.WriteByte('\n')
		}
		return err
	}

	writeSectionHeader := func(header string) error {
		if err := bw.WriteByte('\n'); err != nil {
			return err
		}
		return writeString(header)
	}

	if err := writeString("# Predictive State Specification (PSS)"); err != nil {
		return err
	}
	if err := writeString("version: " + formatValue(c.Version)); err != nil {
		return err
	}
	if c.Persona != "" {
		if err := writeString("persona: " + formatValue(c.Persona)); err != nil {
			return err
		}
	}
	if c.StartState != "" {
		if err := writeString("start: " + formatValue(c.StartState)); err != nil {
			return err
		}
	}
	if c.Goal.Objective != "" {
		if err := writeString("goal: " + formatValue(c.Goal.Objective)); err != nil {
			return err
		}
	}

	// Target BP
	scratch = append(scratch[:0], "target: "...)
	scratch = appendBP(scratch, c.Goal.TargetBP)
	if err := writeBytes(scratch); err != nil {
		return err
	}

	// Tolerance BP
	scratch = append(scratch[:0], "tolerance: "...)
	scratch = appendBP(scratch, c.Goal.TolBP)
	if err := writeBytes(scratch); err != nil {
		return err
	}

	// Current Diff BP
	scratch = append(scratch[:0], "current_difference: "...)
	scratch = appendBP(scratch, c.CurrentDiffBP)
	if err := writeBytes(scratch); err != nil {
		return err
	}

	if err := writeString("zero_knowledge: " + strconv.FormatBool(c.ZeroKnowledge)); err != nil {
		return err
	}

	if len(c.Constraints) > 0 {
		if err := writeSectionHeader("# Constraints"); err != nil {
			return err
		}
		keys := make([]string, 0, len(c.Constraints))
		for k := range c.Constraints {
			keys = append(keys, k)
		}
		sort.Strings(keys)
		for _, k := range keys {
			if err := writeString("constraint: " + formatValue(k) + "=" + formatValue(c.Constraints[k])); err != nil {
				return err
			}
		}
	}

	if len(c.Context) > 0 && !c.ZeroKnowledge {
		if err := writeSectionHeader("# Context"); err != nil {
			return err
		}
		for _, ctxVal := range c.Context {
			if err := writeString("context: " + formatValue(ctxVal)); err != nil {
				return err
			}
		}
	}

	if len(c.Gates) > 0 {
		if err := writeSectionHeader("# Section Gates"); err != nil {
			return err
		}
		for _, gate := range c.Gates {
			scratch = append(scratch[:0], "gate: "...)
			scratch = append(scratch, formatValue(gate.Name)...)
			scratch = append(scratch, " | "...)
			scratch = strconv.AppendBool(scratch, gate.Passed)
			scratch = append(scratch, " | "...)
			scratch = append(scratch, formatValue(gate.Description)...)
			scratch = append(scratch, " | "...)
			scratch = gate.EvaluatedAt.AppendFormat(scratch, time.RFC3339)

			if err := writeBytes(scratch); err != nil {
				return err
			}
		}
	}

	return bw.Flush()
}

// FormatContext converts Capsule to string via bytes.Buffer using the provided Context.
func FormatContext(ctx context.Context, c *capsule.Capsule) (string, error) {
	var buf bytes.Buffer
	if err := FormatWriter(ctx, &buf, c); err != nil {
		return "", err
	}
	return buf.String(), nil
}

// Format converts Capsule to string via bytes.Buffer using context.Background().
func Format(c *capsule.Capsule) (string, error) {
	return FormatContext(context.Background(), c)
}

// SaveFile writes the Capsule specification to disk atomically using standard defer pattern.
func SaveFile(ctx context.Context, c *capsule.Capsule, path string) error {
	if ctx == nil {
		return ErrNilContext
	}

	dir := filepath.Dir(path)
	tmpFile, err := os.CreateTemp(dir, ".pss-tmp-*")
	if err != nil {
		return fmt.Errorf("pss: failed to create temporary file: %w", err)
	}

	tmpName := tmpFile.Name()
	success := false
	defer func() {
		_ = tmpFile.Close()
		if !success {
			_ = os.Remove(tmpName)
		}
	}()

	if err := FormatWriter(ctx, tmpFile, c); err != nil {
		return fmt.Errorf("pss: failed to format output: %w", err)
	}

	if err := tmpFile.Sync(); err != nil {
		return fmt.Errorf("pss: failed to sync temporary file to disk: %w", err)
	}

	if err := os.Rename(tmpName, path); err != nil {
		return fmt.Errorf("pss: failed to atomically replace file: %w", err)
	}

	success = true
	return nil
}

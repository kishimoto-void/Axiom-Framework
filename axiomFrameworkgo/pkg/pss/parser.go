package pss

import (
	"bufio"
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"strconv"
	"time"

	"miniaxiom/pkg/capsule"
)

// ParseReader reads PSS DSL stream from an io.Reader using []byte operations for low-allocation parsing.
func ParseReader(ctx context.Context, r io.Reader, clock capsule.Clock) (*capsule.Capsule, error) {
	if ctx == nil {
		return nil, ErrNilContext
	}
	if clock == nil {
		return nil, ErrNilClock
	}
	if r == nil {
		return nil, ErrNilReader
	}

	c, err := capsule.NewDeterministic("", "", "", clock)
	if err != nil {
		return nil, fmt.Errorf("pss: failed to initialize capsule: %w", err)
	}

	parseTime := clock.Now()
	reader := bufio.NewReader(r)
	lineNum := 0

	for {
		if err := ctx.Err(); err != nil {
			return nil, err
		}

		lineBytes, err := reader.ReadBytes('\n')
		if len(lineBytes) > 0 {
			lineNum++
			if parseErr := parseLine(c, lineBytes, lineNum, parseTime); parseErr != nil {
				return nil, parseErr
			}
		}

		if err != nil {
			if errors.Is(err, io.EOF) {
				break
			}
			return nil, fmt.Errorf("pss: stream read error: %w", err)
		}
	}

	c.Normalize(parseTime)
	if err := c.AssertInvariants(); err != nil {
		return nil, fmt.Errorf("pss: parsed capsule failed invariant check: %w", err)
	}

	return c, nil
}

// ParseContext parses raw PSS syntax string using the provided Context.
func ParseContext(ctx context.Context, input string, clock capsule.Clock) (*capsule.Capsule, error) {
	return ParseReader(ctx, bytes.NewReader([]byte(input)), clock)
}

// Parse parses raw PSS syntax string using context.Background().
func Parse(input string, clock capsule.Clock) (*capsule.Capsule, error) {
	return ParseContext(context.Background(), input, clock)
}

// ParseFile parses a file path using Context.
func ParseFile(ctx context.Context, path string, clock capsule.Clock) (*capsule.Capsule, error) {
	if ctx == nil {
		return nil, ErrNilContext
	}

	f, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("pss: failed to open file %s: %w", path, err)
	}
	defer f.Close()

	return ParseReader(ctx, f, clock)
}

func parseLine(c *capsule.Capsule, lineBytes []byte, lineNum int, parseTime time.Time) error {
	line := bytes.TrimSpace(lineBytes)
	if len(line) == 0 || line[0] == '#' || bytes.HasPrefix(line, []byte("//")) {
		return nil
	}

	keyBytes, valBytes, found := bytes.Cut(line, []byte(":"))
	if !found {
		return &ParseError{Line: lineNum, Err: ErrMissingSeparator}
	}

	keyBytes = bytes.ToLower(bytes.TrimSpace(keyBytes))
	valBytes = bytes.TrimSpace(valBytes)
	keyStr := string(keyBytes)

	// Lazy string extraction for value only when needed
	getValString := func() (string, error) {
		if bytes.HasPrefix(valBytes, []byte("\"")) {
			unquoted, err := strconv.Unquote(string(valBytes))
			if err != nil {
				return "", fmt.Errorf("%w: invalid quoted string: %v", ErrInvalidSyntax, err)
			}
			return unquoted, nil
		}
		return string(valBytes), nil
	}

	switch keyStr {
	case "version":
		v, err := getValString()
		if err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}
		c.Version = v

	case "persona":
		v, err := getValString()
		if err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}
		c.Persona = v

	case "start", "start_state":
		v, err := getValString()
		if err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}
		c.StartState = v

	case "goal", "objective":
		v, err := getValString()
		if err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}
		c.Goal.Objective = v

	case "target", "target_bp":
		bp, err := parseBasisPointBytes(valBytes)
		if err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}
		c.Goal.TargetBP = bp

	case "tolerance", "tolerance_bp":
		bp, err := parseBasisPointBytes(valBytes)
		if err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}
		c.Goal.TolBP = bp

	case "current_difference", "current_difference_bp":
		bp, err := parseBasisPointBytes(valBytes)
		if err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}
		if err := c.UpdateDifference(bp, parseTime); err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}

	case "zero_knowledge":
		v, err := getValString()
		if err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}
		zk, err := strconv.ParseBool(v)
		if err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: fmt.Errorf("%w: %v", ErrInvalidSyntax, err)}
		}
		if zk && len(c.Context) > 0 {
			return &ParseError{Line: lineNum, Key: keyStr, Err: ErrZeroKnowledgeError}
		}
		c.ZeroKnowledge = zk

	case "constraint":
		kBytes, vBytes, hasVal := bytes.Cut(valBytes, []byte("="))
		k := string(bytes.TrimSpace(kBytes))
		v := "true"
		if hasVal {
			v = string(bytes.TrimSpace(vBytes))
		}
		if k == "" {
			return &ParseError{Line: lineNum, Key: keyStr, Err: fmt.Errorf("%w: empty constraint key", ErrInvalidSyntax)}
		}
		if c.Constraints == nil {
			c.Constraints = make(map[string]string)
		}
		c.Constraints[k] = v

	case "context":
		if c.ZeroKnowledge {
			return &ParseError{Line: lineNum, Key: keyStr, Err: ErrZeroKnowledgeError}
		}
		v, err := getValString()
		if err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}
		if v != "" {
			c.Context = append(c.Context, v)
		}

	case "gate":
		gateParts := bytes.Split(valBytes, []byte("|"))
		if len(gateParts) < 1 || len(bytes.TrimSpace(gateParts[0])) == 0 {
			return &ParseError{Line: lineNum, Key: keyStr, Err: fmt.Errorf("%w: gate name required", ErrInvalidSyntax)}
		}
		if len(gateParts) > 4 {
			return &ParseError{Line: lineNum, Key: keyStr, Err: fmt.Errorf("%w: too many gate parameters", ErrInvalidSyntax)}
		}

		name := string(bytes.TrimSpace(gateParts[0]))
		passed := false
		desc := ""
		evalTime := parseTime

		if len(gateParts) > 1 {
			p, err := strconv.ParseBool(string(bytes.ToLower(bytes.TrimSpace(gateParts[1]))))
			if err != nil {
				return &ParseError{Line: lineNum, Key: keyStr, Err: fmt.Errorf("%w: invalid gate passed status: %v", ErrInvalidSyntax, err)}
			}
			passed = p
		}
		if len(gateParts) > 2 {
			desc = string(bytes.TrimSpace(gateParts[2]))
		}
		if len(gateParts) > 3 {
			tStr := string(bytes.TrimSpace(gateParts[3]))
			if tStr != "" {
				parsedTime, err := time.Parse(time.RFC3339, tStr)
				if err != nil {
					return &ParseError{Line: lineNum, Key: keyStr, Err: fmt.Errorf("%w: invalid gate timestamp: %v", ErrInvalidSyntax, err)}
				}
				evalTime = parsedTime
			}
		}

		staticClock, err := capsule.NewMockClock(evalTime)
		if err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}
		if err := c.AddGate(name, passed, desc, staticClock); err != nil {
			return &ParseError{Line: lineNum, Key: keyStr, Err: err}
		}
	}

	return nil
}

func parseBasisPointBytes(val []byte) (capsule.BasisPoint, error) {
	val = bytes.TrimSpace(val)
	if len(val) >= 2 && bytes.EqualFold(val[len(val)-2:], []byte("bp")) {
		clean := bytes.TrimSpace(val[:len(val)-2])
		v, err := strconv.ParseInt(string(clean), 10, 64)
		if err != nil {
			return 0, fmt.Errorf("%w: invalid BP integer: %v", ErrInvalidSyntax, err)
		}
		return capsule.NewBasisPoint(v)
	}

	f, err := strconv.ParseFloat(string(val), 64)
	if err != nil {
		return 0, fmt.Errorf("%w: invalid float value: %v", ErrInvalidSyntax, err)
	}
	return capsule.FloatToBP(f)
}

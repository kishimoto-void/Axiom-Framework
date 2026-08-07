package pss

import (
	"errors"
	"fmt"
)

var (
	ErrInvalidSyntax      = errors.New("pss: invalid syntax")
	ErrMissingSeparator   = errors.New("pss: missing key-value separator ':'")
	ErrZeroKnowledgeError = errors.New("pss: zero-knowledge constraint violation")
	ErrNilClock           = errors.New("pss: clock dependency cannot be nil")
	ErrNilContext         = errors.New("pss: context cannot be nil")
	ErrNilReader          = errors.New("pss: reader cannot be nil")
	ErrNilWriter          = errors.New("pss: writer cannot be nil")
	ErrNilCapsule         = errors.New("pss: capsule cannot be nil")
)

// ParseError represents a structured, inspectable error with precise line location and field context.
type ParseError struct {
	Line int
	Key  string
	Err  error
}

func (e *ParseError) Error() string {
	if e.Key != "" {
		return fmt.Sprintf("pss: parse error at line %d (key %q): %v", e.Line, e.Key, e.Err)
	}
	return fmt.Sprintf("pss: parse error at line %d: %v", e.Line, e.Err)
}

func (e *ParseError) Unwrap() error {
	return e.Err
}

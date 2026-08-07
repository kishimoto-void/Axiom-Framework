package adapter

import (
	"context"
	"errors"
	"fmt"
	"io"
	"sort"
	"sync"
	"unicode/utf8"
)

var (
	ErrAdapterNotFound   = errors.New("adapter not found")
	ErrAlreadyRegistered = errors.New("adapter already registered")
	ErrInvalidSnapshot   = errors.New("invalid snapshot metadata, encoding or format")
	ErrNilRenderer       = errors.New("renderer cannot be nil")
	ErrNilWriter         = errors.New("writer cannot be nil")
	ErrContextCanceled   = errors.New("operation canceled by context")
	ErrExceededLimit     = errors.New("input size, count or depth exceeded safety limit")
)

const (
	// MaxASTDepth limits recursive node nesting to prevent stack exhaustion and runaway generation.
	MaxASTDepth = 16
	// MaxContextItems is the hard upper bound on context entries accepted by Snapshot.Validate.
	MaxContextItems = 1000
	// MaxConstraintsCount is the hard upper bound on constraint key-value pairs.
	MaxConstraintsCount = 1000
	// MaxStringLength is the per-field byte-length safety limit (approx. 1 MiB).
	MaxStringLength = 1_000_000
	// MaxVersionStrLen bounds version identifier strings.
	MaxVersionStrLen = 256
)

// Message is a single chat-style message.
type Message struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

// KeyValue is a simple ordered key-value pair used for constraints and metadata.
type KeyValue struct {
	Key   string `json:"key"`
	Value string `json:"value"`
}

// VersionInfo carries adapter / protocol / schema version identifiers.
type VersionInfo struct {
	AdapterVersion  string `json:"adapter_version"`
	ProtocolVersion string `json:"protocol_version"`
	SchemaVersion   string `json:"schema_version"`
}

// Metadata is attached to the final Payload.
type Metadata struct {
	Adapter string      `json:"adapter"`
	Version VersionInfo `json:"version"`
	Custom  []KeyValue  `json:"custom,omitempty"`
}

// Payload is the final result returned by Format.
type Payload struct {
	SystemPrompt string    `json:"system_prompt"`
	UserPrompt   string    `json:"user_prompt"`
	Messages     []Message `json:"messages,omitempty"`
	Metadata     Metadata  `json:"metadata"`
}

// Snapshot is the structured input that adapters render.
type Snapshot struct {
	Persona     string      `json:"persona"`
	Version     VersionInfo `json:"version"`
	Constraints []KeyValue  `json:"constraints"`
	Context     []string    `json:"context"`
}

// validateTextField validates both byte-length bounds and UTF-8 validity.
// The length check is performed on the raw byte length for predictable resource accounting.
func validateTextField(s, fieldName string, maxLen int) error {
	if len(s) > maxLen {
		return fmt.Errorf("%w: field %s exceeds length limit (%d > %d)", ErrExceededLimit, fieldName, len(s), maxLen)
	}
	if !utf8.ValidString(s) {
		return fmt.Errorf("%w: field %s contains invalid UTF-8 sequence", ErrInvalidSnapshot, fieldName)
	}
	return nil
}

// Validate performs defensive bounds and encoding checks on the Snapshot.
// Empty constraint Values are permitted (useful for boolean-style flags).
// Empty constraint Keys are rejected.
func (s Snapshot) Validate() error {
	if err := validateTextField(s.Persona, "persona", MaxStringLength); err != nil {
		return err
	}

	if err := validateTextField(s.Version.AdapterVersion, "version.adapter_version", MaxVersionStrLen); err != nil {
		return err
	}
	if err := validateTextField(s.Version.ProtocolVersion, "version.protocol_version", MaxVersionStrLen); err != nil {
		return err
	}
	if err := validateTextField(s.Version.SchemaVersion, "version.schema_version", MaxVersionStrLen); err != nil {
		return err
	}

	if len(s.Constraints) > MaxConstraintsCount {
		return fmt.Errorf("%w: constraints count exceeds limit (%d > %d)", ErrExceededLimit, len(s.Constraints), MaxConstraintsCount)
	}
	for i, kv := range s.Constraints {
		if kv.Key == "" {
			return fmt.Errorf("%w: constraint key at index %d cannot be empty", ErrInvalidSnapshot, i)
		}
		if err := validateTextField(kv.Key, fmt.Sprintf("constraints[%d].key", i), MaxStringLength); err != nil {
			return err
		}
		if err := validateTextField(kv.Value, fmt.Sprintf("constraints[%d].value", i), MaxStringLength); err != nil {
			return err
		}
	}

	if len(s.Context) > MaxContextItems {
		return fmt.Errorf("%w: context items count exceeds limit (%d > %d)", ErrExceededLimit, len(s.Context), MaxContextItems)
	}
	for i, item := range s.Context {
		if err := validateTextField(item, fmt.Sprintf("context[%d]", i), MaxStringLength); err != nil {
			return err
		}
	}

	return nil
}

// DeepCopy returns an independent copy of the Snapshot (slices are newly allocated).
func (s Snapshot) DeepCopy() Snapshot {
	constraintsCopy := make([]KeyValue, len(s.Constraints))
	copy(constraintsCopy, s.Constraints)

	contextCopy := make([]string, len(s.Context))
	copy(contextCopy, s.Context)

	return Snapshot{
		Persona:     s.Persona,
		Version:     s.Version,
		Constraints: constraintsCopy,
		Context:     contextCopy,
	}
}

// Adapter is the core interface every concrete adapter must satisfy.
type Adapter interface {
	FormatTo(ctx context.Context, w io.Writer, snap Snapshot, userInput string, opts ...Option) error
	Format(ctx context.Context, snap Snapshot, userInput string, opts ...Option) (*Payload, error)
}

// Registration describes a registered adapter factory.
type Registration struct {
	Name         string
	Version      string
	Priority     int // Higher value = higher selection priority
	Capabilities []string
	Factory      Factory
}

// Validate checks that a Registration is well-formed.
func (r Registration) Validate() error {
	if r.Name == "" {
		return errors.New("registration name cannot be empty")
	}
	if r.Version == "" {
		return errors.New("registration version cannot be empty")
	}
	if r.Factory == nil {
		return errors.New("registration factory cannot be nil")
	}
	for i, cap := range r.Capabilities {
		if cap == "" {
			return fmt.Errorf("registration capability at index %d cannot be empty", i)
		}
	}
	return nil
}

// DeepCopy returns a copy with a newly allocated Capabilities slice.
func (r Registration) DeepCopy() Registration {
	caps := make([]string, len(r.Capabilities))
	copy(caps, r.Capabilities)
	r.Capabilities = caps
	return r
}

// Factory creates a fresh Adapter instance.
type Factory func() Adapter

// Registry is a concurrent-safe catalog of adapters.
type Registry struct {
	mu            sync.RWMutex
	registrations map[string]Registration
}

// NewRegistry returns an empty registry.
func NewRegistry() *Registry {
	return &Registry{
		registrations: make(map[string]Registration),
	}
}

// Register adds a new adapter. Duplicate names are rejected.
func (r *Registry) Register(reg Registration) error {
	if err := reg.Validate(); err != nil {
		return fmt.Errorf("invalid registration: %w", err)
	}

	r.mu.Lock()
	defer r.mu.Unlock()

	if _, exists := r.registrations[reg.Name]; exists {
		return fmt.Errorf("%w: %s", ErrAlreadyRegistered, reg.Name)
	}
	r.registrations[reg.Name] = reg.DeepCopy()
	return nil
}

// New instantiates an adapter by name.
func (r *Registry) New(name string) (Adapter, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()

	reg, exists := r.registrations[name]
	if !exists {
		return nil, fmt.Errorf("%w: %s", ErrAdapterNotFound, name)
	}
	return reg.Factory(), nil
}

// SelectByCapability returns registrations that advertise the given capability,
// sorted by Priority (desc) then Name (asc).
func (r *Registry) SelectByCapability(capability string) []Registration {
	r.mu.RLock()
	defer r.mu.RUnlock()

	var matched []Registration
	for _, reg := range r.registrations {
		for _, cap := range reg.Capabilities {
			if cap == capability {
				matched = append(matched, reg.DeepCopy())
				break
			}
		}
	}

	sort.SliceStable(matched, func(i, j int) bool {
		if matched[i].Priority == matched[j].Priority {
			return matched[i].Name < matched[j].Name
		}
		return matched[i].Priority > matched[j].Priority
	})

	return matched
}

// List returns all registrations sorted by Priority (desc) then Name (asc).
func (r *Registry) List() []Registration {
	r.mu.RLock()
	defer r.mu.RUnlock()

	list := make([]Registration, 0, len(r.registrations))
	for _, reg := range r.registrations {
		list = append(list, reg.DeepCopy())
	}

	sort.SliceStable(list, func(i, j int) bool {
		if list[i].Priority == list[j].Priority {
			return list[i].Name < list[j].Name
		}
		return list[i].Priority > list[j].Priority
	})

	return list
}

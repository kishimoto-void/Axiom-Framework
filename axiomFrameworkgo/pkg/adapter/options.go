package adapter

import "context"

type options struct {
	maxContextItems int
	customMetadata  []KeyValue
	tracer          TracerHook
	skipDeepCopy    bool
}

// TracerHook is an optional observability callback.
// Implementations must be non-blocking and panic-free.
type TracerHook func(ctx context.Context, event string, details map[string]string)

func defaultOptions() options {
	return options{
		maxContextItems: 0, // 0 = unlimited (still subject to Snapshot.Validate hard limit)
		customMetadata:  nil,
		tracer:          nil,
		skipDeepCopy:    false,
	}
}

// Option configures Format / FormatTo behaviour.
type Option func(*options)

// WithMaxContextItems limits the number of context items rendered.
// A value of 0 (or negative, which is clamped) means “use all items that passed Validate”.
func WithMaxContextItems(max int) Option {
	return func(o *options) {
		if max < 0 {
			max = 0
		}
		o.maxContextItems = max
	}
}

// WithMetadata appends a custom key-value pair to the Payload metadata.
// Empty keys are silently ignored to prevent accidental pollution.
func WithMetadata(key, value string) Option {
	return func(o *options) {
		if key == "" {
			return
		}
		cp := make([]KeyValue, len(o.customMetadata), len(o.customMetadata)+1)
		copy(cp, o.customMetadata)
		o.customMetadata = append(cp, KeyValue{Key: key, Value: value})
	}
}

// WithTracer installs an observability hook.
func WithTracer(tracer TracerHook) Option {
	return func(o *options) {
		o.tracer = tracer
	}
}

// WithUnsafeNoCopy explicitly signals that the caller guarantees the input Snapshot
// will not be modified concurrently or during execution.
// Using this option skips the defensive DeepCopy and is therefore faster,
// but places the responsibility for immutability on the caller.
func WithUnsafeNoCopy() Option {
	return func(o *options) {
		o.skipDeepCopy = true
	}
}

// WithNoCopy is an alias for WithUnsafeNoCopy.
//
// Deprecated: Prefer WithUnsafeNoCopy for explicit intent.
func WithNoCopy() Option {
	return WithUnsafeNoCopy()
}

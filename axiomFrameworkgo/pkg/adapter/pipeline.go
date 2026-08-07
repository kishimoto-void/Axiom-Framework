package adapter

import (
	"bytes"
	"context"
	"fmt"
	"io"
)

// BaseAdapter provides the common Format / FormatTo implementation
// used by concrete adapters (OpenAI, Claude, …).
type BaseAdapter struct {
	Name        string
	Renderer    Renderer
	WrapPayload func(sysPrompt, userInput string) []Message
}

// FormatTo renders the snapshot directly into the supplied writer.
func (b *BaseAdapter) FormatTo(ctx context.Context, w io.Writer, snap Snapshot, userInput string, opts ...Option) error {
	if w == nil {
		return ErrNilWriter
	}
	cfg := defaultOptions()
	for _, opt := range opts {
		if opt != nil {
			opt(&cfg)
		}
	}
	return b.formatToWithOptions(ctx, w, snap, userInput, cfg)
}

func (b *BaseAdapter) formatToWithOptions(ctx context.Context, w io.Writer, snap Snapshot, userInput string, cfg options) error {
	if ctx != nil {
		if err := ctx.Err(); err != nil {
			return fmt.Errorf("%w: %v", ErrContextCanceled, err)
		}
	}
	if b.Renderer == nil {
		return ErrNilRenderer
	}

	if err := validateTextField(userInput, "userInput", MaxStringLength); err != nil {
		return err
	}

	if err := snap.Validate(); err != nil {
		return err
	}

	if cfg.tracer != nil {
		cfg.tracer(ctx, "format_to_start", map[string]string{"adapter": b.Name})
	}

	targetSnap := snap
	if !cfg.skipDeepCopy {
		targetSnap = snap.DeepCopy()
	}

	ast, err := buildGenericAST(ctx, targetSnap, cfg)
	if err != nil {
		return err
	}

	if err := b.Renderer.RenderDocument(ctx, w, ast, targetSnap.Persona, targetSnap.Version, cfg.tracer); err != nil {
		return err
	}

	if cfg.tracer != nil {
		cfg.tracer(ctx, "format_to_end", map[string]string{"adapter": b.Name})
	}

	return nil
}

// Format renders the snapshot into a Payload value.
func (b *BaseAdapter) Format(ctx context.Context, snap Snapshot, userInput string, opts ...Option) (*Payload, error) {
	cfg := defaultOptions()
	for _, opt := range opts {
		if opt != nil {
			opt(&cfg)
		}
	}

	var buf bytes.Buffer
	buf.Grow(1024)

	if err := b.formatToWithOptions(ctx, &buf, snap, userInput, cfg); err != nil {
		return nil, err
	}

	rendered := buf.String()

	var messages []Message
	if b.WrapPayload != nil {
		messages = b.WrapPayload(rendered, userInput)
	}

	var customMetaCopy []KeyValue
	if len(cfg.customMetadata) > 0 {
		customMetaCopy = make([]KeyValue, len(cfg.customMetadata))
		copy(customMetaCopy, cfg.customMetadata)
	}

	return &Payload{
		SystemPrompt: rendered,
		UserPrompt:   userInput,
		Messages:     messages,
		Metadata: Metadata{
			Adapter: b.Name,
			Version: snap.Version, // Version is a pure value type; no need to copy
			Custom:  customMetaCopy,
		},
	}, nil
}

// buildGenericAST constructs a simple two-section AST from Constraints and Context.
// Empty-title root is intentional so that child sections start at depth 1.
func buildGenericAST(ctx context.Context, snap Snapshot, cfg options) (*Node, error) {
	if ctx != nil {
		if err := ctx.Err(); err != nil {
			return nil, fmt.Errorf("%w: %v", ErrContextCanceled, err)
		}
	}

	root := &Node{Type: NodeSection, Title: ""}

	if len(snap.Constraints) > 0 {
		root.Children = append(root.Children, &Node{
			Type:  NodeSection,
			Title: "Constraints",
			Children: []*Node{
				{Type: NodeKV, KV: snap.Constraints},
			},
		})
	}

	ctxItems := snap.Context
	if len(ctxItems) > 0 {
		if cfg.maxContextItems > 0 && len(ctxItems) > cfg.maxContextItems {
			truncated := make([]string, cfg.maxContextItems)
			copy(truncated, ctxItems[:cfg.maxContextItems])
			ctxItems = truncated
		}
		root.Children = append(root.Children, &Node{
			Type:  NodeSection,
			Title: "Context",
			Children: []*Node{
				{Type: NodeList, List: ctxItems},
			},
		})
	}

	return root, nil
}

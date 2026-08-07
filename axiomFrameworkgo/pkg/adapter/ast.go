package adapter

import (
	"context"
	"encoding/json"
	"fmt"
)

// NodeType classifies AST nodes.
type NodeType int

const (
	NodeSection NodeType = iota
	NodeKV
	NodeList
	NodeText
)

// Node is a lightweight intermediate representation used by renderers.
type Node struct {
	Type     NodeType   `json:"type"`
	Title    string     `json:"title,omitempty"`
	Value    string     `json:"value,omitempty"`
	KV       []KeyValue `json:"kv,omitempty"`
	List     []string   `json:"list,omitempty"`
	Children []*Node    `json:"children,omitempty"`
}

// Visitor is the double-dispatch interface for AST traversal.
type Visitor interface {
	VisitEnter(ctx context.Context, node *Node, depth int) error
	VisitLeave(ctx context.Context, node *Node, depth int) error
}

// Accept starts a depth-first walk of the AST.
func (n *Node) Accept(ctx context.Context, v Visitor) error {
	if n == nil || v == nil {
		return nil
	}
	return n.acceptDepth(ctx, v, 0)
}

func (n *Node) acceptDepth(ctx context.Context, v Visitor, depth int) error {
	if depth > MaxASTDepth {
		return fmt.Errorf("%w: AST depth exceeds maximum allowed level %d", ErrExceededLimit, MaxASTDepth)
	}

	if ctx != nil {
		if err := ctx.Err(); err != nil {
			return fmt.Errorf("%w: %v", ErrContextCanceled, err)
		}
	}

	if err := v.VisitEnter(ctx, n, depth); err != nil {
		return err
	}

	// Empty-title sections are depth-transparent so that nested content
	// inherits the parent depth (useful for synthetic root nodes).
	nextDepth := depth + 1
	if n.Type == NodeSection && n.Title == "" {
		nextDepth = depth
	}

	for _, child := range n.Children {
		if child == nil {
			continue
		}
		if ctx != nil {
			if err := ctx.Err(); err != nil {
				return fmt.Errorf("%w: %v", ErrContextCanceled, err)
			}
		}
		if err := child.acceptDepth(ctx, v, nextDepth); err != nil {
			return err
		}
	}

	if ctx != nil {
		if err := ctx.Err(); err != nil {
			return fmt.Errorf("%w: %v", ErrContextCanceled, err)
		}
	}

	return v.VisitLeave(ctx, n, depth)
}

// ToJSON returns a pretty-printed JSON representation (primarily for debugging).
func (n *Node) ToJSON() ([]byte, error) {
	return json.MarshalIndent(n, "", "  ")
}

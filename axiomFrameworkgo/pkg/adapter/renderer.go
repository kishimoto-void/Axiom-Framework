package adapter

import (
	"context"
	"fmt"
	"io"
	"strconv"
	"strings"
)

// Renderer produces the final textual representation of an AST.
//
// Contract:
//   - Callers must ensure that all string fields passed to RenderDocument
//     are already validated for UTF-8 (Snapshot.Validate / validateTextField).
//   - Renderers are responsible for format-specific escaping, layout,
//     and (for XML) control-character rejection.
type Renderer interface {
	RenderDocument(ctx context.Context, w io.Writer, ast *Node, persona string, version VersionInfo, tracer TracerHook) error
}

// High-risk Markdown character escaper.
// Escapes structural break characters while preserving natural prose readability.
var markdownEscaper = strings.NewReplacer(
	"\\", "\\\\",
	"`", "\\`",
	"*", "\\*",
	"_", "\\_",
	"[", "\\[",
	"]", "\\]",
	"#", "\\#",
	"|", "\\|",
)

func escapeMarkdown(input string) string {
	return markdownEscaper.Replace(input)
}

// sanitizeCommentToken sanitizes metadata strings that will be embedded
// inside HTML/XML comments.  Replacing "--" with the em-dash "—" prevents
// accidental or malicious premature comment termination (e.g. "-->" injection).
func sanitizeCommentToken(s string) string {
	return strings.ReplaceAll(s, "--", "—")
}

// MarkdownRenderer emits a human-readable Markdown prompt.
type MarkdownRenderer struct{}

// NewMarkdownRenderer returns a ready-to-use Markdown renderer.
func NewMarkdownRenderer() Renderer { return &MarkdownRenderer{} }

func (r *MarkdownRenderer) RenderDocument(ctx context.Context, w io.Writer, ast *Node, persona string, version VersionInfo, tracer TracerHook) error {
	if w == nil {
		return ErrNilWriter
	}

	if version.AdapterVersion != "" || version.ProtocolVersion != "" || version.SchemaVersion != "" {
		if _, err := fmt.Fprintf(
			w,
			"<!-- Version: adapter=%s, protocol=%s, schema=%s -->\n",
			sanitizeCommentToken(version.AdapterVersion),
			sanitizeCommentToken(version.ProtocolVersion),
			sanitizeCommentToken(version.SchemaVersion),
		); err != nil {
			return err
		}
	}

	if persona != "" {
		if _, err := fmt.Fprintf(w, "You are operating as: %s.\n\n", escapeMarkdown(persona)); err != nil {
			return err
		}
	}

	if ast != nil {
		v := &markdownVisitor{w: w, tracer: tracer}
		if err := ast.Accept(ctx, v); err != nil {
			return err
		}
	}
	return nil
}

type markdownVisitor struct {
	w      io.Writer
	tracer TracerHook
}

func (v *markdownVisitor) VisitEnter(ctx context.Context, node *Node, depth int) error {
	if ctx != nil {
		if err := ctx.Err(); err != nil {
			return fmt.Errorf("%w: %v", ErrContextCanceled, err)
		}
	}

	switch node.Type {
	case NodeSection:
		if node.Title != "" {
			effectiveDepth := depth
			if effectiveDepth < 1 {
				effectiveDepth = 1
			} else if effectiveDepth > 6 {
				if v.tracer != nil {
					v.tracer(ctx, "depth_capped", map[string]string{
						"original_depth": strconv.Itoa(depth),
						"capped_depth":   "6",
						"section_title":  node.Title,
					})
				}
				effectiveDepth = 6
			}
			prefix := strings.Repeat("#", effectiveDepth)
			if _, err := fmt.Fprintf(v.w, "%s %s\n", prefix, escapeMarkdown(node.Title)); err != nil {
				return err
			}
		}
	case NodeText:
		if node.Value != "" {
			if _, err := fmt.Fprintf(v.w, "%s\n", escapeMarkdown(node.Value)); err != nil {
				return err
			}
		}
	case NodeKV:
		for _, kv := range node.KV {
			if ctx != nil {
				if err := ctx.Err(); err != nil {
					return fmt.Errorf("%w: %v", ErrContextCanceled, err)
				}
			}
			if _, err := fmt.Fprintf(v.w, "- %s: %s\n", escapeMarkdown(kv.Key), escapeMarkdown(kv.Value)); err != nil {
				return err
			}
		}
	case NodeList:
		for _, item := range node.List {
			if ctx != nil {
				if err := ctx.Err(); err != nil {
					return fmt.Errorf("%w: %v", ErrContextCanceled, err)
				}
			}
			if _, err := fmt.Fprintf(v.w, "- %s\n", escapeMarkdown(item)); err != nil {
				return err
			}
		}
	}
	return nil
}

func (v *markdownVisitor) VisitLeave(ctx context.Context, node *Node, depth int) error {
	if ctx != nil {
		if err := ctx.Err(); err != nil {
			return fmt.Errorf("%w: %v", ErrContextCanceled, err)
		}
	}
	if node.Type == NodeSection && node.Title != "" {
		if _, err := io.WriteString(v.w, "\n"); err != nil {
			return err
		}
	}
	return nil
}

// ---------------------------------------------------------------------------
// XML 1.0 character validation (kept inside the renderer package boundary)
// ---------------------------------------------------------------------------

// isValidXMLRune implements the XML 1.0 Char production:
// #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
func isValidXMLRune(r rune) bool {
	return r == 0x09 || r == 0x0A || r == 0x0D ||
		(r >= 0x20 && r <= 0xD7FF) ||
		(r >= 0xE000 && r <= 0xFFFD) ||
		(r >= 0x10000 && r <= 0x10FFFF)
}

// validateXMLText rejects any rune that is illegal in XML 1.0 text content.
func validateXMLText(s, fieldName string) error {
	for i, r := range s {
		if !isValidXMLRune(r) {
			return fmt.Errorf("%w: field %s contains invalid XML control character (U+%04X) at byte index %d",
				ErrInvalidSnapshot, fieldName, r, i)
		}
	}
	return nil
}

var xmlEscaper = strings.NewReplacer(
	"&", "&",
	"<", "<",
	">", ">",
	"\"", """,
	"'", "'",
)

func xmlEscape(input string) string {
	return xmlEscaper.Replace(input)
}

// XMLRenderer emits a well-formed XML document suitable for Claude-style system prompts.
type XMLRenderer struct{}

// NewXMLRenderer returns a ready-to-use XML renderer.
func NewXMLRenderer() Renderer { return &XMLRenderer{} }

func (r *XMLRenderer) RenderDocument(ctx context.Context, w io.Writer, ast *Node, persona string, version VersionInfo, tracer TracerHook) error {
	if w == nil {
		return ErrNilWriter
	}

	// Reject XML 1.0 illegal control characters before any output is written.
	if err := validateXMLText(version.AdapterVersion, "version.adapter_version"); err != nil {
		return err
	}
	if err := validateXMLText(version.ProtocolVersion, "version.protocol_version"); err != nil {
		return err
	}
	if err := validateXMLText(version.SchemaVersion, "version.schema_version"); err != nil {
		return err
	}
	if err := validateXMLText(persona, "persona"); err != nil {
		return err
	}

	if _, err := fmt.Fprintf(
		w,
		"<capsule adapter_version=\"%s\" protocol_version=\"%s\" schema_version=\"%s\">\n",
		xmlEscape(version.AdapterVersion),
		xmlEscape(version.ProtocolVersion),
		xmlEscape(version.SchemaVersion),
	); err != nil {
		return err
	}

	if persona != "" {
		if _, err := fmt.Fprintf(w, "  <persona>%s</persona>\n", xmlEscape(persona)); err != nil {
			return err
		}
	}

	if ast != nil {
		v := &xmlVisitor{w: w, tracer: tracer}
		if err := ast.Accept(ctx, v); err != nil {
			return err
		}
	}

	if _, err := io.WriteString(w, "</capsule>\n"); err != nil {
		return err
	}
	return nil
}

type xmlVisitor struct {
	w      io.Writer
	tracer TracerHook
}

func (v *xmlVisitor) VisitEnter(ctx context.Context, node *Node, depth int) error {
	if ctx != nil {
		if err := ctx.Err(); err != nil {
			return fmt.Errorf("%w: %v", ErrContextCanceled, err)
		}
	}

	pad := strings.Repeat("  ", depth+1)
	switch node.Type {
	case NodeSection:
		if node.Title != "" {
			if err := validateXMLText(node.Title, "section.title"); err != nil {
				return err
			}
			if _, err := fmt.Fprintf(v.w, "%s<section name=\"%s\">\n", pad, xmlEscape(node.Title)); err != nil {
				return err
			}
		}
	case NodeText:
		if node.Value != "" {
			if err := validateXMLText(node.Value, "text.value"); err != nil {
				return err
			}
			if _, err := fmt.Fprintf(v.w, "%s<text>%s</text>\n", pad, xmlEscape(node.Value)); err != nil {
				return err
			}
		}
	case NodeKV:
		for _, kv := range node.KV {
			if ctx != nil {
				if err := ctx.Err(); err != nil {
					return fmt.Errorf("%w: %v", ErrContextCanceled, err)
				}
			}
			if err := validateXMLText(kv.Key, "kv.key"); err != nil {
				return err
			}
			if err := validateXMLText(kv.Value, "kv.value"); err != nil {
				return err
			}
			if _, err := fmt.Fprintf(v.w, "%s<item key=\"%s\">%s</item>\n", pad, xmlEscape(kv.Key), xmlEscape(kv.Value)); err != nil {
				return err
			}
		}
	case NodeList:
		for _, item := range node.List {
			if ctx != nil {
				if err := ctx.Err(); err != nil {
					return fmt.Errorf("%w: %v", ErrContextCanceled, err)
				}
			}
			if err := validateXMLText(item, "list.item"); err != nil {
				return err
			}
			if _, err := fmt.Fprintf(v.w, "%s<item>%s</item>\n", pad, xmlEscape(item)); err != nil {
				return err
			}
		}
	}
	return nil
}

func (v *xmlVisitor) VisitLeave(ctx context.Context, node *Node, depth int) error {
	if ctx != nil {
		if err := ctx.Err(); err != nil {
			return fmt.Errorf("%w: %v", ErrContextCanceled, err)
		}
	}
	if node.Type == NodeSection && node.Title != "" {
		pad := strings.Repeat("  ", depth+1)
		if _, err := fmt.Fprintf(v.w, "%s</section>\n", pad); err != nil {
			return err
		}
	}
	return nil
}

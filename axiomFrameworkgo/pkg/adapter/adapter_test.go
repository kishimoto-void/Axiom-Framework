package adapter_test

import (
	"bytes"
	"context"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"miniaxiom/pkg/adapter"
)

func newMockSnapshot() adapter.Snapshot {
	return adapter.Snapshot{
		Persona: "System Architect",
		Version: adapter.VersionInfo{
			AdapterVersion:  "v1.0.0--alpha",
			ProtocolVersion: "v1",
			SchemaVersion:   "2026-08",
		},
		Constraints: []adapter.KeyValue{
			{Key: "a_perf", Value: "Latency < 100ms & Fast!"},
			{Key: "b_sec", Value: "<script>alert(1)</script>"},
		},
		Context: []string{"Context Item 1", "Context Item 2"},
	}
}

func TestGoldenFiles(t *testing.T) {
	ctx := context.Background()
	snap := newMockSnapshot()

	tests := []struct {
		name       string
		adapter    adapter.Adapter
		goldenFile string
	}{
		{
			name:       "Claude_XML",
			adapter:    adapter.NewClaude(),
			goldenFile: "claude_output.golden",
		},
		{
			name:       "OpenAI_Markdown",
			adapter:    adapter.NewOpenAI(),
			goldenFile: "openai_output.golden",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			payload, err := tt.adapter.Format(ctx, snap, "User input prompt")
			if err != nil {
				t.Fatalf("Format failed: %v", err)
			}

			goldenPath := filepath.Join("testdata", tt.goldenFile)

			if os.Getenv("UPDATE_GOLDEN") == "true" {
				_ = os.MkdirAll("testdata", 0755)
				if err := os.WriteFile(goldenPath, []byte(payload.SystemPrompt), 0644); err != nil {
					t.Fatalf("Failed to update golden file: %v", err)
				}
			}

			expected, err := os.ReadFile(goldenPath)
			if err != nil {
				t.Skipf("Golden file %s not found, run with UPDATE_GOLDEN=true to create", goldenPath)
			}

			if string(expected) != payload.SystemPrompt {
				t.Errorf("Golden test mismatch for %s!\nExpected:\n%s\nGot:\n%s", tt.name, string(expected), payload.SystemPrompt)
			}
		})
	}
}

func TestUserInputValidation(t *testing.T) {
	ctx := context.Background()
	snap := newMockSnapshot()
	adp := adapter.NewOpenAI()

	// Test invalid UTF-8 in userInput
	invalidInput := string([]byte{0xff, 0xfe})
	_, err := adp.Format(ctx, snap, invalidInput)
	if !errors.Is(err, adapter.ErrInvalidSnapshot) {
		t.Errorf("Expected ErrInvalidSnapshot for invalid UTF-8 in userInput, got %v", err)
	}
}

func TestMarkdownControlCharAllowed(t *testing.T) {
	ctx := context.Background()
	snap := newMockSnapshot()
	snap.Persona = "Developer\x07WithBell"

	adp := adapter.NewOpenAI()
	payload, err := adp.Format(ctx, snap, "Input")
	if err != nil {
		t.Fatalf("Expected Markdown adapter to succeed with control char, got %v", err)
	}
	if !strings.Contains(payload.SystemPrompt, "Developer\x07WithBell") {
		t.Errorf("Expected control character in Markdown prompt output")
	}
}

func TestXMLControlCharRejectedAtRender(t *testing.T) {
	ctx := context.Background()
	snap := newMockSnapshot()
	snap.Persona = "Developer\x07WithBell"

	adp := adapter.NewClaude()
	_, err := adp.Format(ctx, snap, "Input")
	if !errors.Is(err, adapter.ErrInvalidSnapshot) {
		t.Errorf("Expected ErrInvalidSnapshot for XML with control char, got %v", err)
	}
}

func TestEmptyTitleDepthTransparent(t *testing.T) {
	ctx := context.Background()
	root := &adapter.Node{
		Type:  adapter.NodeSection,
		Title: "",
		Children: []*adapter.Node{
			{
				Type:  adapter.NodeSection,
				Title: "NestedTitle",
			},
		},
	}

	var buf bytes.Buffer
	renderer := adapter.NewMarkdownRenderer()
	err := renderer.RenderDocument(ctx, &buf, root, "", adapter.VersionInfo{}, nil)
	if err != nil {
		t.Fatalf("RenderDocument failed: %v", err)
	}

	if !strings.HasPrefix(buf.String(), "# NestedTitle") {
		t.Errorf("Expected depth 1 (# NestedTitle) for transparent empty title, got:\n%s", buf.String())
	}
}

func TestRegistrySecondarySort(t *testing.T) {
	reg := adapter.NewRegistry()

	_ = reg.Register(adapter.Registration{Name: "b_adapter", Version: "v1", Priority: 10, Factory: adapter.NewOpenAI})
	_ = reg.Register(adapter.Registration{Name: "a_adapter", Version: "v1", Priority: 10, Factory: adapter.NewClaude})

	list := reg.List()
	if len(list) != 2 {
		t.Fatalf("Expected 2 items, got %d", len(list))
	}

	if list[0].Name != "a_adapter" || list[1].Name != "b_adapter" {
		t.Errorf("Secondary sort failed! Expected [a_adapter, b_adapter], got [%s, %s]", list[0].Name, list[1].Name)
	}
}

func TestCommentTokenSanitization(t *testing.T) {
	ctx := context.Background()
	snap := newMockSnapshot()
	snap.Version.AdapterVersion = "v1.0--preview-->"

	adp := adapter.NewOpenAI()
	payload, err := adp.Format(ctx, snap, "")
	if err != nil {
		t.Fatalf("Format failed: %v", err)
	}

	if strings.Contains(payload.SystemPrompt, "--preview") {
		t.Errorf("HTML Comment contains unescaped '--' token!\n%s", payload.SystemPrompt)
	}
	if !strings.Contains(payload.SystemPrompt, "—preview—>") {
		t.Errorf("Expected '--' to be sanitized to '—', got:\n%s", payload.SystemPrompt)
	}
}

func TestNilWriterRejected(t *testing.T) {
	ctx := context.Background()
	snap := newMockSnapshot()
	adp := adapter.NewOpenAI()

	err := adp.FormatTo(ctx, nil, snap, "input")
	if !errors.Is(err, adapter.ErrNilWriter) {
		t.Errorf("Expected ErrNilWriter, got %v", err)
	}
}

type XMLCapsuleDoc struct {
	XMLName         xml.Name `xml:"capsule"`
	AdapterVersion  string   `xml:"adapter_version,attr"`
	ProtocolVersion string   `xml:"protocol_version,attr"`
	SchemaVersion   string   `xml:"schema_version,attr"`
	Persona         string   `xml:"persona"`
}

// FuzzPipelineDeterministic tests Format(), FormatTo(), and RenderDocument()
// across randomly generated inputs to guarantee:
// 1. No panics occur.
// 2. Deterministic behavior (identical input always yields identical output).
// 3. Output equivalence between Format() and FormatTo().
func FuzzPipelineDeterministic(f *testing.F) {
	f.Add("PersonaText", "KeyName", "ValContent", "ContextDetail", "v1.0", "p1", "s1", "UserInputText")
	f.Fuzz(func(t *testing.T, persona, key, val, ctxItem, vAdapt, vProto, vSchema, userInput string) {
		snap := adapter.Snapshot{
			Persona: persona,
			Version: adapter.VersionInfo{
				AdapterVersion:  vAdapt,
				ProtocolVersion: vProto,
				SchemaVersion:   vSchema,
			},
			Constraints: []adapter.KeyValue{
				{Key: key, Value: val},
			},
			Context: []string{ctxItem},
		}

		adapters := []adapter.Adapter{
			adapter.NewOpenAI(),
			adapter.NewClaude(),
		}

		ctx := context.Background()

		for _, adp := range adapters {
			// First run with Format()
			payload1, err1 := adp.Format(ctx, snap, userInput)

			// Second run with FormatTo()
			var buf bytes.Buffer
			err2 := adp.FormatTo(ctx, &buf, snap, userInput)

			// Both execution methods must yield identical error status
			if (err1 == nil) != (err2 == nil) {
				t.Fatalf("Format and FormatTo error mismatch! Format err: %v, FormatTo err: %v", err1, err2)
			}

			if err1 != nil {
				continue
			}

			// Output equivalence check between Format() and FormatTo()
			if payload1.SystemPrompt != buf.String() {
				t.Fatalf("Output mismatch between Format and FormatTo!\nFormat output:\n%s\nFormatTo output:\n%s", payload1.SystemPrompt, buf.String())
			}

			// Determinism check: Run a second time with exact same inputs
			payload3, err3 := adp.Format(ctx, snap, userInput)
			if err3 != nil {
				t.Fatalf("Second Format run failed unexpectedly: %v", err3)
			}

			if payload1.SystemPrompt != payload3.SystemPrompt {
				t.Fatalf("Non-deterministic output detected!\nRun 1:\n%s\nRun 2:\n%s", payload1.SystemPrompt, payload3.SystemPrompt)
			}

			// Validate XML structure if output is XML
			if strings.HasPrefix(payload1.SystemPrompt, "<capsule") {
				var doc XMLCapsuleDoc
				if err := xml.Unmarshal([]byte(payload1.SystemPrompt), &doc); err != nil {
					t.Fatalf("Generated XML is invalid!\nOutput:\n%s\nError: %v", payload1.SystemPrompt, err)
				}
			}
		}
	})
}

func BenchmarkSnapshotValidate(b *testing.B) {
	snap := newMockSnapshot()
	for i := 0; i < 100; i++ {
		snap.Constraints = append(snap.Constraints, adapter.KeyValue{Key: fmt.Sprintf("key_%d", i), Value: "value_string_sample"})
	}
	for i := 0; i < 500; i++ {
		snap.Context = append(snap.Context, fmt.Sprintf("Context Item line content %d", i))
	}

	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		_ = snap.Validate()
	}
}

func BenchmarkDeepCopy(b *testing.B) {
	snap := newMockSnapshot()
	for i := 0; i < 100; i++ {
		snap.Constraints = append(snap.Constraints, adapter.KeyValue{Key: fmt.Sprintf("key_%d", i), Value: "value_string_sample"})
	}
	for i := 0; i < 500; i++ {
		snap.Context = append(snap.Context, fmt.Sprintf("Context Item line content %d", i))
	}

	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		_ = snap.DeepCopy()
	}
}

func BenchmarkMarkdownRenderer(b *testing.B) {
	ctx := context.Background()
	snap := newMockSnapshot()
	renderer := adapter.NewMarkdownRenderer()

	ast := &adapter.Node{
		Type:  adapter.NodeSection,
		Title: "BenchmarkSection",
		Children: []*adapter.Node{
			{Type: adapter.NodeKV, KV: snap.Constraints},
			{Type: adapter.NodeList, List: snap.Context},
		},
	}

	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		_ = renderer.RenderDocument(ctx, io.Discard, ast, snap.Persona, snap.Version, nil)
	}
}

func BenchmarkXMLRenderer(b *testing.B) {
	ctx := context.Background()
	snap := newMockSnapshot()
	renderer := adapter.NewXMLRenderer()

	ast := &adapter.Node{
		Type:  adapter.NodeSection,
		Title: "BenchmarkSection",
		Children: []*adapter.Node{
			{Type: adapter.NodeKV, KV: snap.Constraints},
			{Type: adapter.NodeList, List: snap.Context},
		},
	}

	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		_ = renderer.RenderDocument(ctx, io.Discard, ast, snap.Persona, snap.Version, nil)
	}
}

func BenchmarkFormatToStream(b *testing.B) {
	ctx := context.Background()
	snap := newMockSnapshot()
	adp := adapter.NewOpenAI()

	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		_ = adp.FormatTo(ctx, io.Discard, snap, "Benchmark Input", adapter.WithUnsafeNoCopy())
	}
}

// Package pss provides deterministic parsing and formatting for Predictive State Specification (PSS) DSL streams.
//
// Key Specifications:
//   - Context Support: Stream processing supports cancellation via context.Context (ParseContext, FormatContext).
//   - Zero-Knowledge Mode: If "zero_knowledge: true" is encountered, subsequent "context:" directives
//     will return an ErrZeroKnowledgeError. DSL execution is strictly sequential.
//   - Gate Directives: "gate: name | passed | desc | rfc3339_time". Default for passed is false if omitted.
//   - Quoting: Values containing delimiters (':', '|', '#'), newlines, or whitespace are automatically quoted.
//   - Determinism: Clock dependencies are injected explicitly, guaranteeing reproducible outputs.
package pss

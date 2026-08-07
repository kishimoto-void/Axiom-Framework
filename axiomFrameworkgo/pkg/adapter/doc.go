// Package adapter provides a deterministic, security-conscious rendering pipeline
// that converts structured Snapshot data into LLM-compatible prompts
// (Markdown for OpenAI-style, XML for Claude-style).
//
// Design principles:
//   - Determinism: identical inputs always produce identical outputs.
//   - Safety limits: depth, item count, and string length bounds are enforced.
//   - Escape hygiene: Markdown and XML escaping + HTML comment token sanitization
//     prevent structural injection and control-character issues.
//   - Zero-copy option: WithUnsafeNoCopy allows callers that guarantee immutability
//     to skip DeepCopy for performance-critical paths.
//   - Context cancellation: all long-running paths respect context.Context.
package adapter

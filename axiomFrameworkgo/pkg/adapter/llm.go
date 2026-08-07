package adapter

// NewOpenAI returns an adapter that produces Markdown system prompts
// and wraps them in a standard OpenAI-style message list
// (system + optional user).
func NewOpenAI() Adapter {
	return &BaseAdapter{
		Name:     "openai",
		Renderer: NewMarkdownRenderer(),
		WrapPayload: func(sysPrompt, userInput string) []Message {
			msgs := []Message{{Role: "system", Content: sysPrompt}}
			if userInput != "" {
				msgs = append(msgs, Message{Role: "user", Content: userInput})
			}
			return msgs
		},
	}
}

// NewClaude returns an adapter that produces XML system prompts.
// Claude-style APIs typically accept the system prompt as a top-level field,
// therefore only the user message is placed in the Messages slice.
func NewClaude() Adapter {
	return &BaseAdapter{
		Name:     "claude",
		Renderer: NewXMLRenderer(),
		WrapPayload: func(sysPrompt, userInput string) []Message {
			msgs := make([]Message, 0, 1)
			if userInput != "" {
				msgs = append(msgs, Message{Role: "user", Content: userInput})
			}
			return msgs
		},
	}
}

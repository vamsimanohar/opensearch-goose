# traces.conversation

Group all spans tagged with a GenAI conversation id
(`attributes.gen_ai.conversation.id`) and render them as a chronological
timeline grouped by traceId (each trace is one turn). Pass `dataSourceId` and
`conversationId`. Optional `dataset` (default `otel-v1-apm-span-*`) and
`limit` (default 500).

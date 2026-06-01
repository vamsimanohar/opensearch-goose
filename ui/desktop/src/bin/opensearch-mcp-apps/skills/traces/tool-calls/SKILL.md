# traces.tool-calls

Inspect all GenAI tool-call spans in a single trace (where
`attributes.gen_ai.tool.name` is set). Renders one card per tool call with
name, call id, arguments, and result. Pass `dataSourceId` and `traceId`.
Optional `dataset` (default `otel-v1-apm-span-*`) and `limit` (default 500).

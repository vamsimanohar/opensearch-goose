# traces.gantt

Render a Gantt-style timeline of all spans for a trace, sorted by start time
(flat, not hierarchical). Each span is a horizontal bar coloured by service.
Pass `dataSourceId` and `traceId`. Optional `dataset` (defaults to
`otel-v1-apm-span-*`) and `limit` (default 500). Use when the user wants to
inspect concurrency and ordering of spans rather than parent/child structure.

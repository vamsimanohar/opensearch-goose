# traces.cross-signal-join

Join spans and logs by traceId (and spanId where available in logs) and render
them as a single chronological timeline. Use when the user wants to see
"everything that happened" for a trace — not just spans and not just logs.
Pass `dataSourceId`, `logsDataset`, `traceId`. Optional `spansDataset`
(default `otel-v1-apm-span-*`) and `limit` (default 500 per side).

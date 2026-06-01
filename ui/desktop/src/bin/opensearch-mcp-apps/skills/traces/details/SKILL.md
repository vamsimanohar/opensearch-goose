# traces.details

Render the spans for a single trace as a hierarchy + timeline view, with a
side panel that shows the selected span's overview and raw attributes.

## When to invoke

- The user provides a trace ID and asks to inspect / explore / debug a trace.
- The user wants to see the parent/child structure of spans, durations,
  service breakdown, or attributes for a specific trace.

## Arguments

- `traceId` (required) — the OpenTelemetry trace ID.
- `dataset` (optional) — index/data-stream pattern. Defaults to
  `otel-v1-apm-span-*`. Override when traces live elsewhere
  (e.g. `traces-otel-*`).
- `limit` (optional) — max spans to load. Default `500`.

## When NOT to invoke

- When the user wants a custom dashboard, chart, or aggregation — call
  `canvas_generate` instead.
- When no `traceId` is known. Either ask the user, or have them call
  `canvas_generate` to find one first.

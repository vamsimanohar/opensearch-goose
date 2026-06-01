# traces.root-only

List trace root spans (where `parentSpanId = ""`), sorted by `startTime`
descending. Pass `dataSourceId`. Optional `dataset` (default
`otel-v1-apm-span-*`) and `limit` (default 100). Use to find recent traces
by their root operation.

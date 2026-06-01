# spans.span-links

List spans that have one or more `links` (cross-trace references) and show
each target traceId/spanId. Pass `dataSourceId`. Optional `dataset` (default
`otel-v1-apm-span-*`), `traceId`, and `limit` (default 100). Falls back to
`links != ""` if `array_length` is unsupported.

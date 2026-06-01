# traces.dag

Render the spans of a single trace as a node-and-edge DAG (parent→child) using
inline SVG. Use when the user wants to see the topology of a trace at a glance,
not the timeline. Pass `dataSourceId` and `traceId`. Optional `dataset`
(defaults to `otel-v1-apm-span-*`) and `limit` (default 500).

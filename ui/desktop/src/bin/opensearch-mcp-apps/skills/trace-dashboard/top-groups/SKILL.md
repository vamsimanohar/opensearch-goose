# trace-dashboard.top-groups

Rank arbitrary group-by attributes (e.g. `resource.environment`,
`attributes.app.version`, `serviceName`) by trace count and p99 latency. Pass
`dataSourceId` and `groupBy`. Optional `dataset` (default `otel-v1-apm-span-*`)
and `limit` (default 20). Use to spot which dimension (env, version, region…)
is dominating volume or driving latency.

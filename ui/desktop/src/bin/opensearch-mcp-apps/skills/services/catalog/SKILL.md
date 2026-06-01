# services.catalog

Use this when the user wants a sortable list of services with their RED-style KPIs (span count, error rate, avg/p50/p95/p99 latency). Computes from a trace span index. Pass `dataSourceId` and `dataset` (the span index pattern, e.g. `otel-v1-apm-span-*`).

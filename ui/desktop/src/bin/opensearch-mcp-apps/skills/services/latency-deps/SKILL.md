# services.latency-deps

Use when the user wants per-dependency latency for one service (p50/p90/p99). Pass `dataSourceId`, `serviceName`, optional `dataset` (defaults to `otel-v1-apm-span-*`) and `percentile` (`50`/`90`/`99`).

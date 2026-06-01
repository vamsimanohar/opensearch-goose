# apm-red.compute

Use when the user wants RED metrics (rate, errors, latency percentiles) for a single service, bucketed over time. Pass `dataSourceId`, `serviceName`, optional `dataset` (defaults to `otel-v1-apm-span-*`) and `bucketSize` (`1m` | `5m` | `1h`).

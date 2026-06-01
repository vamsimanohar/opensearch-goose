# trace-dashboard.top-services

Use when the user wants a leaderboard of services by p99 latency, error rate, or throughput from a trace span index. Pass `dataSourceId`, optionally `dataset` (defaults to `otel-v1-apm-span-*`) and `metric` (`latency` | `error` | `throughput`).

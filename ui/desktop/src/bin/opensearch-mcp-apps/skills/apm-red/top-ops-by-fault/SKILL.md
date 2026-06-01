## apm-red.top-ops-by-fault

Rank a service's operations by fault rate to identify the worst offenders inside one service. Pass `dataSourceId` and `serviceName`; optional `dataset` (defaults `otel-v1-apm-span-*`) and `limit` (default 10). Returns one row per operation: total spans, error spans, fault rate.

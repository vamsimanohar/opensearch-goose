## apm-traces.error-types

Group error spans (statusCode = 2) for a service by `attributes.error.type` and count occurrences. Pass `dataSourceId` and `serviceName`; optional `dataset` (defaults `otel-v1-apm-span-*`) and `limit` (default 20). One row per error type with count.

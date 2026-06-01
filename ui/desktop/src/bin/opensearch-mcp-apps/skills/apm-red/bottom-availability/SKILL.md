## apm-red.bottom-availability

List the bottom-K services by availability (`availability = 1 - fault_rate`). Pass `dataSourceId`; optional `dataset` (defaults `otel-v1-apm-span-*`) and `limit` (default 10). One row per service with total spans, errors, fault rate and availability — sorted ascending so the worst services come first.

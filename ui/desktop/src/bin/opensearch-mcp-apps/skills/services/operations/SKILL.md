# services.operations

Use when the user wants to browse the operations of a service with sortable per-op KPIs (calls, errors, error rate, avg/p50/p95/p99 latency). Pass `dataSourceId`, `serviceName`, optional `dataset`, `filter` (`minLatencyMs`, `maxErrorRate`, `search`), and `limit`. Also covers "list service operations from the service map".

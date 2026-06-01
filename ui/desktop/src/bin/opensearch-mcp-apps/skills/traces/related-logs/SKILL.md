# traces.related-logs

Load logs linked to a specific trace (and optionally a single span) via the
`traceId` / `spanId` log fields. Pass `dataSourceId`, `logsDataset`, and
`traceId`. Optional `spanId` and `limit` (default 200). Use to bridge a trace
to its application log lines.

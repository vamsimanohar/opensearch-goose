## metrics-view.latest-values

List the latest value per series for a single metric. Pass `dataSourceId`, `metricName`, optional `dataset` (defaults `prometheus.metrics`) and `limit`. Returns one row per `labels` group with the most recent value and timestamp; if PPL fails the failed query is surfaced for review.

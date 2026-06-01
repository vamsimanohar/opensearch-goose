# log-patterns.change-detection

Use to compare log patterns between an alert window and a baseline window.
Pass `dataSourceId`, `dataset`, `patternsField`, and the four ISO 8601
timestamps `alertStart`/`alertEnd`/`baselineStart`/`baselineEnd`. Renders a
table sorted by absolute delta — useful for finding what's new or surging
during an alert window.

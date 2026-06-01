# log-patterns.rare

Use to surface the rarest recurring log patterns (the inverse of `top`). Pass
`dataSourceId`, `dataset`, and `patternsField` (typically `body`, `message`,
or `log`). Optional `filter` (PPL chain like `where severity = "ERROR"`),
`method` (`brain` (default) or `regex`), and `limit` (default 30). Useful for
finding rare or anomalous patterns hiding under common noise.

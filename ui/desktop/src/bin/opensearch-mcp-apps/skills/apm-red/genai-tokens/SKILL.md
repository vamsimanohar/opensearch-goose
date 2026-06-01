## apm-red.genai-tokens

Aggregate GenAI input / output token usage by model or agent. Pass `dataSourceId`; optional `dataset` (defaults `otel-v1-apm-span-*`), `groupBy` (`model` (default) | `agent`), and `limit` (default 10). Returns one row per group with input, output, and total tokens.

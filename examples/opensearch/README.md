# OpenSearch Recipes

Goose recipes for working with OpenSearch clusters and the OTel observability stack.

## Recipes

| Recipe | Use when |
|---|---|
| [`investigate-traces.yaml`](./investigate-traces.yaml) | Querying OTel trace data — slow spans, errors, agent invocations, token usage |
| [`search-logs.yaml`](./search-logs.yaml) | Querying OTel log data — severity filters, trace correlation, error patterns |
| [`build-ppl-query.yaml`](./build-ppl-query.yaml) | Composing, validating, and explaining PPL queries against any index |

## Setup

Set these environment variables before running a recipe:

```bash
export OPENSEARCH_URL=https://localhost:9200
export OPENSEARCH_USERNAME=admin
export OPENSEARCH_PASSWORD='your-password'
# AWS managed: set AWS_REGION + AWS_PROFILE instead
```

The bundled `opensearch` extension uses the
[opensearch-mcp-server-py](https://github.com/opensearch-project/opensearch-mcp-server-py)
MCP server, invoked via `uvx`. Install [uv](https://docs.astral.sh/uv/) once:

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

The MCP server itself fetches automatically on first run.

## Run a recipe

```bash
goose run --recipe examples/opensearch/investigate-traces.yaml
```

## Index patterns assumed

These recipes target the default OTel index patterns:

| Signal | Index pattern |
|---|---|
| Traces | `otel-v1-apm-span-*` |
| Logs | `logs-otel-v1-*` |

Override via recipe parameters if your stack uses different patterns.

## Credit

PPL query templates adapted from
[opensearch-project/observability-stack/claude-code-observability-plugin](https://github.com/opensearch-project/observability-stack/tree/main/claude-code-observability-plugin).

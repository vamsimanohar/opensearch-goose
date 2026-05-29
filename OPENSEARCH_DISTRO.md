# OpenSearch Goose

A custom distribution of [goose](https://github.com/aaif-goose/goose) tailored for
OpenSearch users — cluster operations, query authoring, and observability
investigations across traces, logs, and metrics.

## What's bundled

1. **[opensearch-mcp-server-py](https://github.com/opensearch-project/opensearch-mcp-server-py)**
   pre-registered as a built-in extension. Provides `ListIndexTool`,
   `IndexMappingTool`, `SearchIndexTool`, `GetShardsTool`, `ClusterHealthTool`,
   `CountTool`, `ExplainTool`, `MsearchTool`, and a `GenericOpenSearchApiTool`.

2. **OpenSearch-flavored system prompt** — agent defaults to OpenSearch idioms
   (PPL field escaping, OpenSearch DSL, non-destructive operations).

3. **Three pre-built recipes** in [`examples/opensearch/`](./examples/opensearch/):
   - `investigate-traces.yaml` — slow spans, errors, agent invocations, token usage
   - `search-logs.yaml` — severity filters, trace correlation, error patterns
   - `build-ppl-query.yaml` — compose, validate, and explain PPL queries

   PPL templates adapted from
   [opensearch-project/observability-stack/claude-code-observability-plugin](https://github.com/opensearch-project/observability-stack/tree/main/claude-code-observability-plugin).

## Quick start

### Prerequisites

- Rust toolchain (handled by `bin/activate-hermit`)
- Node.js / pnpm for the desktop UI
- [`uv`](https://docs.astral.sh/uv/) for running the OpenSearch MCP server

### Build

```bash
source bin/activate-hermit
cargo build --release
just run-ui      # launches desktop with goosed sidecar
```

### Configure your cluster

Set environment variables before launching, or via the in-app extension settings:

```bash
# Local / self-hosted OpenSearch
export OPENSEARCH_URL=https://localhost:9200
export OPENSEARCH_USERNAME=admin
export OPENSEARCH_PASSWORD='your-password'
export OPENSEARCH_SSL_VERIFY=false   # for local dev with self-signed certs

# Amazon OpenSearch Service (SigV4)
export OPENSEARCH_URL=https://your-domain.us-east-1.es.amazonaws.com
export AWS_REGION=us-east-1
export AWS_PROFILE=your-profile
```

### Run a recipe (CLI)

```bash
goose run --recipe examples/opensearch/investigate-traces.yaml
```

## What's left to do

This is a minimal end-to-end fork. Likely follow-ups:

- Replace icons (`ui/desktop/src/images/`)
- Swap deb/rpm package `name` from `Goose` to `OpenSearch-Goose`
- Add Bedrock as the default provider (currently inherits goose defaults)
- Add more recipes (cluster health diagnosis, query profiling, ISM setup)
- Disable or repoint PostHog telemetry (`GOOSE_DISABLE_TELEMETRY=1`)

## Trademark and license

This is an Apache 2.0 fork of goose. "Goose" is a trademark of AAIF; we use it
in the name "OpenSearch Goose" only to indicate this distribution's origin and
do not claim affiliation with or endorsement by AAIF. "OpenSearch" is a
trademark of the OpenSearch Software Foundation; this distribution is not
affiliated with or endorsed by the OpenSearch project.

# OpenSearch Goose

A custom distribution of [goose](https://github.com/aaif-goose/goose) tailored for
OpenSearch users — cluster operations, query authoring, and observability
investigations across traces, logs, and metrics.

## What's bundled

1. **opensearch-mcp-apps** (multi-app MCP server) — pre-registered as a
   bundled `node`-based stdio extension. Connects via SigV4 to an OpenSearch UI
   application and exposes 200+ tools spanning traces, logs, services, alerts,
   dashboards, agent observability, anomaly detection, SLOs, and more. Bundled
   under `ui/desktop/src/bin/opensearch-mcp-apps/`.

2. **AWS IAM Identity Center (IDC) sign-in** — first-class onboarding option.
   The user enters their IDC Start URL and region, picks an account/role, and
   the temporary credentials are written to the goose config as
   `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` / `AWS_SESSION_TOKEN`. Both the
   Bedrock provider and the OpenSearch MCP receive the credentials via the
   standard AWS env-var chain, so a single sign-in unlocks the model and all
   AWS-backed extensions.

3. **OpenSearch-flavored system prompt** — agent defaults to OpenSearch idioms
   (PPL field escaping, OpenSearch DSL, non-destructive operations).

4. **Three pre-built recipes** in [`examples/opensearch/`](./examples/opensearch/):
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

Two options for AWS authentication:

**Option A: Sign in with IDC (recommended)**

1. Launch the desktop app (`just run-ui`).
2. On the welcome screen, click **"Sign in with AWS"**.
3. Enter your IAM Identity Center Start URL (e.g.
   `https://d-xxxxxxxxxx.awsapps.com/start`) and AWS Region.
4. The browser opens to the IDC login page; the verification code is auto-copied
   to your clipboard. Paste it and approve.
5. Pick an AWS account and role.

That's it — the model provider (Bedrock) and the OpenSearch MCP both get the
temporary credentials. They auto-expire in 1 hour and are refreshed via the
SSO access token (8h default).

**Option B: Standard AWS credential chain**

If you already have working AWS credentials (env vars, `~/.aws/credentials`,
`aws sso login --profile X`, etc.) just configure Bedrock as a regular provider
and set the OpenSearch UI endpoint:

```bash
export OPENSEARCH_UI_ENDPOINT=application-foo-bar.us-west-2.opensearch.amazonaws.com
export AWS_REGION=us-west-2
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

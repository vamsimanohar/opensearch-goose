# _template.hello

A minimal MCP App route used to validate the framework contract. Invokes a
trivial server-side handler and renders the returned greeting.

## When to invoke

- The user explicitly asks to test the template app, or to verify that the
  MCP App framework is wired up.
- Pass an optional `name` argument; it gets echoed back in the greeting.

## When NOT to invoke

- For real OpenSearch work — use the domain-specific apps (canvas, alerts,
  logs, etc.) instead.

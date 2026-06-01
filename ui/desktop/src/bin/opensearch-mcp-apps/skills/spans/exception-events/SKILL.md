# spans.exception-events

List spans that contain an `exception` event, with one card per span showing
exception.type / exception.message / exception.stacktrace from the event
attributes. Pass `dataSourceId`. Optional `dataset` (default
`otel-v1-apm-span-*`), `serviceName`, and `limit` (default 100).

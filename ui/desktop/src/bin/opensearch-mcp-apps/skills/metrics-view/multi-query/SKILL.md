## metrics-view.multi-query

Reference card for rendering multiple PromQL expressions on one chart. Pass `dataSourceId` and `queries: string[]`. PromQL execution isn't wired in this build, so the tool stages queries side-by-side and documents the `or` / `and` / `unless` / arithmetic / comparison operators used to compose them.

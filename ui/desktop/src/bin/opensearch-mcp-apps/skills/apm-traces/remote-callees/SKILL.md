# apm-traces.remote-callees

List the remote services a service calls into, ranked by call count, with errors and p99 latency. Coalesces across `peer.service`, `server.address`, `net.peer.name`, `http.host`, `db.name`, `messaging.destination.name`, and `rpc.service`. Invoke when the user asks "who does X talk to?", "what are X's downstream callees?", or "what services does X depend on?".

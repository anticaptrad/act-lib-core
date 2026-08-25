# act-lib-core

Core Anticaptrad domain and persistence library. It imports the canonical Rust
types from `act-interfaces`, validates bounded YouTube control commands, and
provides a SeaORM-backed public result projection.

The projection can be constructed only with an explicit read-only database
capability and issues one named, parameterized `SELECT`. Writes belong to the
API service and its durable workflow boundary; this crate does not expose a
generic query or mutation surface.

Shared Auth establishes identity at the server edge, but its claims are not
product authorization. Consumers must use the official Shared Auth client or
guard, require the exact Anticaptrad audience and scopes, and then apply product
and resource policy before calling this library. User tokens and service
credentials never enter the command contracts or persistence API.

Telemetry fields pass through the pinned `ores-otel/ores-lib-core` redaction
contract. Authorization headers, cookies, tokens, private payloads, database
URLs, and raw upstream errors must not be logged.

Dependencies are declared in both Cargo metadata and `.zpkg.toml`. Cargo uses
immutable Git revisions for `act-interfaces` and `ores-lib-core`; Zed is the
fleet package graph and install authority.

```sh
/path/to/zed validate
/path/to/zed install
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
```

If the Zed registry is unavailable, keep the manifest and Cargo lock intact,
report the registry failure, and do not fabricate `.zpkg.lock` content.

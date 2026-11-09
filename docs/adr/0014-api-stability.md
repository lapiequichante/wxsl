# 0014. Define and check API stability

Status: Accepted

## Context

Consumers need a clear distinction between stable entry points and implementation details.

## Decision

Treat crate-root re-exports as the stable Rust surface and module paths as unstable. Track the public surface with snapshots; version documents and the C ABI separately. Record stable surface changes in the changelog.

## Consequences

Intentional API changes require updating snapshots and compatibility documentation.

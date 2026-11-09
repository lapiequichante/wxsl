# 0002. Workspace boundaries and facade features

Status: Accepted

## Context

Consumers need the graph model independently of a renderer, editor, or standard node library.

## Decision

Keep separate crates for the model, language, frame description, renderer, editor and standard library. The facade exposes optional capabilities through additive features. `wxsl-frame` depends on `wxsl-core` but not on a device, renderer, editor or standard library.

## Consequences

Consumers can choose a smaller dependency graph. Cross-crate dependencies must preserve the boundaries documented in the architecture.

# 0008. Effects and bakes are declared units

Status: Accepted

## Context

Hardcoded pass enums and shader parameters prevent applications from composing new effects.

## Decision

Declare effects with their inputs, outputs, shader and parameters. Support screen-domain graphs and compute effects; represent bakes as effects over material subgraphs. Expand multi-pass recipes as scoped document nodes.

## Consequences

Applications can register effects without changing renderer dispatch; descriptors and shaders must agree.

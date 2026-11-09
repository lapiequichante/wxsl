# 0022. Bake precomputed radiance transfer offline

Status: Accepted

## Context

Environment lighting needs visibility and bounce information for static geometry without tracing it every frame.

## Decision

Use `wxsl-prt` to bake transfer data offline and apply it as environment occlusion through material graphs.

## Consequences

Baking is a separate CPU workflow; the runtime consumes the resulting data and retains dynamic environment lighting.

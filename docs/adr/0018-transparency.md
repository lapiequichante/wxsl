# 0018. Use tiered transparency and optical layers

Status: Accepted

## Context

Transparent and refractive surfaces require different tradeoffs between cost and ordering accuracy.

## Decision

Use sorted transparency by default and dual depth peeling when requested, bounded by the material layer count. Compose optical layers back to front and represent transmission and thin-walled behavior as optical-layer data.

## Consequences

Applications select an appropriate quality and cost; pass ordering and layer limits affect the result.

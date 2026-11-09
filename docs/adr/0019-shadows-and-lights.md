# 0019. Represent lights and shadows as frame data

Status: Accepted

## Context

A frame may contain shadowed lights and many more unshadowed ranged lights while operating at large world coordinates.

## Decision

Pack shadow views in a shared depth atlas and configure its size in the pipeline. Bin unshadowed ranged lights into a host-built grid. Rebase positions on a floating origin and allow static shadows to be cached.

## Consequences

Shadow atlas capacity, light-grid bounds and coordinate conversion are shared frame responsibilities.

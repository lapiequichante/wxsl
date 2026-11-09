# 0005. Compile a material graph for each stage

Status: Accepted

## Context

A single material must work consistently across shading, depth, shadows and other passes.

## Decision

Compile the reachable graph for each material stage. Stage analysis places nodes and creates interpolants; fragment-only constraints are checked. Derive resources and required geometry from the graph.

## Consequences

Passes can share material authoring, while stage and interface errors are reported before drawing.

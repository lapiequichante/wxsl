# 0023. Render planar reflections with a mirrored view

Status: Accepted

## Context

A planar reflector needs to capture geometry outside the main camera view on raster backends.

## Decision

Declare the reflection plane in the device-free model and render a clipped mirrored camera view. Composite the capture on the depth-masked receiver.

## Consequences

The method works with ordinary raster passes and is limited to a configured plane.

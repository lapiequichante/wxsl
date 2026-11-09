# 0020. Represent environment lighting as frame resources

Status: Accepted

## Context

Environment lighting and local probes need explicitly scheduled captures and filtered textures.

## Decision

Represent cubes, mip levels and attachment subresources as frame resources. Use face passes for environment prefiltering and capture local probes in the frame for per-pass lighting.

## Consequences

Scheduling and resource lifetimes govern environment bakes and captures; backends consume the same descriptions.

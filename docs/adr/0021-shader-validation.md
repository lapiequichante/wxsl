# 0021. Validate generated shaders before release

Status: Accepted

## Context

Generated shaders span many stages, models and device profiles; host compilation alone cannot establish shader validity.

## Decision

Validate generated shaders against a representative corpus and gate releases on shipped shader compilation through the C ABI for each supported profile. GPU tests use error scopes for device diagnostics.

## Consequences

Changes to generators, shader sources or profiles must preserve the corpus and release checks.

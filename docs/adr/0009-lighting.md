# 0009. Share lighting between rendering paths

Status: Accepted

## Context

Forward and deferred rendering should share lighting behavior while supporting multiple models and bounded attachment budgets.

## Decision

Generate shading and G-buffer handling from the enabled lighting models. Dispatch deferred lighting by model id and share model data in a channel. Allow materials to shade forward under a deferred pipeline. Keep radiance linear until the tonemap effect.

## Consequences

Generated shaders must be validated and forward/deferred behavior checked for parity.

# 0024. Simulate and draw particles through declared effects

Status: Accepted

## Context

Applications need persistent GPU particles without adding simulation or emitter-specific behavior to the renderer.

## Decision

Provide `effect::particles` as two shipped effects over one stable storage buffer. `particles::append` adds a compute simulation followed by additive, camera-facing sprites that read opaque depth for soft intersections. The host supplies emitter parameters, time and time step through the ordinary effect parameter API.

Each system owns 4096 slots, each holding position, age, velocity and lifetime. Slots respawn on a deterministic schedule without counters or atomics. Separate emitters use separate systems and pass labels. Both effects require the native profile because they use compute and vertex-stage storage reads.

## Consequences

Particles persist when an emitter moves. Every slot is drawn, including inactive slots whose quads are moved off screen. The implementation supports emissive particles such as flames and sparks; smoke would require another blending strategy. Particles do not contribute to motion vectors, environment probes or cached reflections.

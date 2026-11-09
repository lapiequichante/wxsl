# 0013. Select rendering capabilities from the device

Status: Accepted

## Context

Some devices cannot use compute or storage buffers, while native targets benefit from them.

## Decision

Describe device capabilities as a profile. Use supported fallback paths where available and report unsupported effects by name. Validate shipped shaders through the C ABI across profiles.

## Consequences

Native features remain available; fallback compatibility and release shader coverage require explicit checks.

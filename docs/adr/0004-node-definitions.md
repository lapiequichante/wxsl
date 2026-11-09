# 0004. Derive node definitions from WXSL

Status: Accepted

## Context

Separate shader functions and node descriptors can disagree about types, sockets and defaults.

## Decision

Derive node definitions from the signature, type bounds and comments of each `.wxsl` source. Arithmetic operations use generic nodes whose types resolve per instance.

## Consequences

Shader authors maintain one source of truth; build-time parsing and validation are required.

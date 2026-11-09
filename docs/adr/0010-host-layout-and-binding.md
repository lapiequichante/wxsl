# 0010. Share host layouts and bind group conventions

Status: Accepted

## Context

Host and shader layouts must agree across Rust, C and WXSL.

## Decision

Allocate four bind groups by update frequency: frame, material, user and pass. Compute or generate shared layouts from their authoritative definitions instead of maintaining independent mirrors.

## Consequences

Layout changes require regenerating affected outputs and checking ABI parity.

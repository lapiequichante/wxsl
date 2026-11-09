# 0011. Share device-free functionality through a data-only C ABI

Status: Accepted

## Context

A Dawn backend needs the same compiler, scheduler and layouts as wgpu without duplicating their rules.

## Decision

Expose versioned compilation and scheduling operations through `wxsl-ffi` as data, without device handles or renderer state. Dawn implements the device half and consumes offline exports or the runtime ABI.

## Consequences

The ABI needs explicit ownership and diagnostics; backend parity must be tested.

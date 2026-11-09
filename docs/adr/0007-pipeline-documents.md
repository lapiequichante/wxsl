# 0007. Pipelines and scheduling are data

Status: Accepted

## Context

Applications need customizable pipelines and validation without coupling scheduling to a GPU backend.

## Decision

Represent pipelines as documents compiled into a device-free `FrameGraph` of passes and resources. Schedule and validate dependencies in `wxsl-frame`. Pass policies govern execution; non-default policies write stable resources. Draw sorting is specified per pass.

## Consequences

Presets are editable documents and scheduling can be tested without a device; each backend must implement the resulting descriptions.

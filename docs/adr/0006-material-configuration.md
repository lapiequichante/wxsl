# 0006. Resolve material configuration once

Status: Accepted

## Context

Material options and identifiers must agree across documents, code generation and rendering.

## Decision

Represent material configuration as `MaterialConfig` and resolve names to identifiers in one place. Namespace registry identifiers and version authored documents.

## Consequences

Configuration and compatibility checks have a shared interpretation across consumers.

# 1. Separate the API and applications

Status: Accepted

## Context

The reusable API is public. The website and Drive application have a separate, private lifecycle and assets.

## Decision

WXSL contains reusable crates, examples and backend implementations. The separate `wxsl_site` repository contains the website and Drive. Applications consume WXSL as Cargo dependencies; no library depends on an application. During initial development the repositories are siblings and Drive uses path dependencies. After the API is published to Git, pin all WXSL dependencies to the same revision.

## Consequences

Each repository builds and tests independently. Cross-repository changes require coordinated updates.

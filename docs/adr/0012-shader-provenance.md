# 0012. Keep provenance for permissive shader ports

Status: Accepted

## Context

A shader library needs clear licensing and traceability for third-party code.

## Decision

Allow ports from audited MIT, Apache-2.0 and BSD-2/3-Clause sources after checking the exact source and pinned revision. Preserve required notices and document provenance and modifications. Review other licenses explicitly before copying.

## Consequences

Contributors must maintain source-level attribution and both applicable NOTICE files.

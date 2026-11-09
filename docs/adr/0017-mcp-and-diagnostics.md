# 0017. Expose editor commands and diagnostics to agents

Status: Accepted

## Context

An agent needs the same editor capabilities as a user, with bounded file access and reliable vocabulary.

## Decision

Build MCP tools over editor commands and queries, confine paths to the project root, and serve calls through the editor host. Generate the node reference and test the authoring guide. Provide generic frame and pixel inspection.

## Consequences

New agent capabilities should first be expressed as editor commands or queries; diagnostic resources must stay synchronized.

# 0016. Use one editor and one mutation path

Status: Accepted

## Context

Material, pipeline and scene editing need consistent behavior, undo and programmatic access.

## Decision

The editor draws itself with the renderer and exposes canvas modes within one editor. Document mutations go through serializable `Command`s applied by `Editor::apply`; undo restores snapshots and queries are read-only.

## Consequences

UI gestures and automation share the same command semantics; new mutations require commands.

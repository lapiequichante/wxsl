# 0003. Own the shading language and compiler

Status: Accepted

## Context

Shader composition needs imports, conditional compilation and generic functions beyond WGSL.

## Decision

Use WXSL, compiled to WGSL by `wxsl-lang`. Resolve imports before monomorphizing templates; reuse the compiler lexer for editor highlighting.

## Consequences

The project maintains its own language implementation and must test its generated WGSL.
